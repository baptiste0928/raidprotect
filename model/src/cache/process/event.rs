//! Update the cache based on incoming event data.

use async_trait::async_trait;
use tracing::error;
use twilight_model::{
    gateway::payload::incoming::{
        ChannelCreate, ChannelDelete, ChannelUpdate, GuildCreate, GuildDelete, GuildUpdate,
        MemberAdd, MemberUpdate, RoleCreate, RoleDelete, RoleUpdate, ThreadCreate, ThreadDelete,
        ThreadUpdate, UnavailableGuild,
    },
    id::{marker::ApplicationMarker, Id},
};

use crate::cache::{
    model::{CachedChannel, CachedGuild, CachedRole, CurrentMember},
    RedisClient, RedisModel,
};

/// Update the cache based on event data.
///
/// This trait is implemented for all Discord event types that are used to keep
/// the cache up-to-date.
#[async_trait]
pub trait UpdateCache {
    /// Name of the event.
    ///
    /// This is used for logging purpose in case of caching error.
    const NAME: &'static str;

    /// Update the cache based on event data.
    ///
    /// If an old value of the updated entry is present in the cache, it will be
    /// returned.
    async fn update(
        &self,
        redis: &RedisClient,
        current_user: Id<ApplicationMarker>,
    ) -> Result<(), anyhow::Error>;
}

#[async_trait]
impl UpdateCache for GuildCreate {
    const NAME: &'static str = "GuildCreate";

    async fn update(
        &self,
        redis: &RedisClient,
        current_user: Id<ApplicationMarker>,
    ) -> Result<(), anyhow::Error> {
        let mut pipe = redis::pipe();
        super::resource::cache_guild(&mut pipe, current_user, &self.0)?;

        let mut conn = redis.conn().await?;
        pipe.query_async(&mut *conn).await?;

        Ok(())
    }
}

#[async_trait]
impl UpdateCache for GuildDelete {
    const NAME: &'static str = "GuildDelete";

    async fn update(
        &self,
        redis: &RedisClient,
        _current_user: Id<ApplicationMarker>,
    ) -> Result<(), anyhow::Error> {
        if let Some(guild) = redis.get::<CachedGuild>(&self.id).await? {
            // Remove all channels and roles from the cache.
            let mut conn = redis.conn().await?;
            let mut pipe = redis::pipe();
            pipe.del(CachedGuild::key_from(&self.id));

            for channel in &guild.channels {
                pipe.del(CachedChannel::key_from(channel));
            }
            for role in &guild.roles {
                pipe.del(CachedRole::key_from(role));
            }

            pipe.query_async(&mut *conn).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl UpdateCache for UnavailableGuild {
    const NAME: &'static str = "UnavailableGuild";

    async fn update(
        &self,
        redis: &RedisClient,
        _current_user: Id<ApplicationMarker>,
    ) -> Result<(), anyhow::Error> {
        if let Some(mut guild) = redis.get::<CachedGuild>(&self.id).await? {
            guild.unavailable = true;

            // Remove all channels and roles from the cache.
            let mut conn = redis.conn().await?;
            let mut pipe = redis::pipe();
            pipe.set(guild.key(), guild.serialize_model()?);

            for channel in &guild.channels {
                pipe.del(CachedChannel::key_from(channel));
            }
            for role in &guild.roles {
                pipe.del(CachedRole::key_from(role));
            }

            pipe.query_async(&mut *conn).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl UpdateCache for GuildUpdate {
    const NAME: &'static str = "GuildUpdate";

    async fn update(
        &self,
        redis: &RedisClient,
        _current_user: Id<ApplicationMarker>,
    ) -> Result<(), anyhow::Error> {
        if let Some(mut guild) = redis.get::<CachedGuild>(&self.id).await? {
            guild.name = self.name.clone();
            guild.icon = self.icon;
            guild.owner_id = self.owner_id;
            redis.set(&guild).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl UpdateCache for ChannelCreate {
    const NAME: &'static str = "ChannelCreate";

    async fn update(
        &self,
        redis: &RedisClient,
        _current_user: Id<ApplicationMarker>,
    ) -> Result<(), anyhow::Error> {
        if let Some(guild_id) = self.guild_id {
            if let Some(mut guild) = redis.get::<CachedGuild>(&guild_id).await? {
                let mut pipe = redis::pipe();
                let mut conn = redis.conn().await?;

                match super::resource::cache_guild_channel(&mut pipe, self) {
                    Ok(_) => {
                        guild.channels.insert(self.id);
                        pipe.set(guild.key(), guild.serialize_model()?);
                    }
                    Err(error) => {
                        error!(error = ?error, "failed to cache guild channel");
                    }
                };

                pipe.query_async(&mut *conn).await?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl UpdateCache for ChannelDelete {
    const NAME: &'static str = "ChannelDelete";

    async fn update(
        &self,
        redis: &RedisClient,
        _current_user: Id<ApplicationMarker>,
    ) -> Result<(), anyhow::Error> {
        let mut pipe = redis::pipe();

        // Remove the channel from the guild.
        if let Some(guild_id) = self.guild_id {
            if let Some(mut guild) = redis.get::<CachedGuild>(&guild_id).await? {
                guild.channels.remove(&self.id);
                pipe.set(guild.key(), guild.serialize_model()?);
            }
        }

        // Remove the channel from the cache.
        pipe.del(CachedChannel::key_from(&self.id));

        let mut conn = redis.conn().await?;
        pipe.query_async(&mut *conn).await?;

        Ok(())
    }
}

#[async_trait]
impl UpdateCache for ChannelUpdate {
    const NAME: &'static str = "ChannelUpdate";

    async fn update(
        &self,
        redis: &RedisClient,
        _current_user: Id<ApplicationMarker>,
    ) -> Result<(), anyhow::Error> {
        if self.guild_id.is_none() {
            return Ok(()); // Ensure the channel is in a guild.
        }

        let mut pipe = redis::pipe();
        let mut conn = redis.conn().await?;

        match super::resource::cache_guild_channel(&mut pipe, self) {
            Ok(_) => pipe.query_async(&mut *conn).await?,
            Err(error) => {
                error!(error = ?error, "failed to cache guild channel");
            }
        }

        Ok(())
    }
}

#[async_trait]
impl UpdateCache for ThreadCreate {
    const NAME: &'static str = "ThreadCreate";

    async fn update(
        &self,
        redis: &RedisClient,
        _current_user: Id<ApplicationMarker>,
    ) -> Result<(), anyhow::Error> {
        if let Some(guild_id) = self.guild_id {
            if let Some(mut guild) = redis.get::<CachedGuild>(&guild_id).await? {
                let mut pipe = redis::pipe();
                let mut conn = redis.conn().await?;

                match super::resource::cache_guild_channel(&mut pipe, self) {
                    Ok(_) => {
                        guild.channels.insert(self.id);
                        pipe.set(guild.key(), guild.serialize_model()?);
                    }
                    Err(error) => {
                        error!(error = ?error, "failed to cache guild channel");
                    }
                };

                pipe.query_async(&mut *conn).await?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl UpdateCache for ThreadDelete {
    const NAME: &'static str = "ThreadDelete";

    async fn update(
        &self,
        redis: &RedisClient,
        _current_user: Id<ApplicationMarker>,
    ) -> Result<(), anyhow::Error> {
        let mut pipe = redis::pipe();

        // Remove the channel from the guild.
        if let Some(mut guild) = redis.get::<CachedGuild>(&self.guild_id).await? {
            guild.channels.remove(&self.id);
            pipe.set(guild.key(), guild.serialize_model()?);
        }

        // Remove the channel from the cache.
        pipe.del(CachedChannel::key_from(&self.id));

        let mut conn = redis.conn().await?;
        pipe.query_async(&mut *conn).await?;

        Ok(())
    }
}

#[async_trait]
impl UpdateCache for ThreadUpdate {
    const NAME: &'static str = "ThreadUpdate";

    async fn update(
        &self,
        redis: &RedisClient,
        _current_user: Id<ApplicationMarker>,
    ) -> Result<(), anyhow::Error> {
        if self.guild_id.is_none() {
            return Ok(()); // Ensure the channel is in a guild.
        }

        let mut pipe = redis::pipe();
        let mut conn = redis.conn().await?;

        match super::resource::cache_guild_channel(&mut pipe, self) {
            Ok(_) => pipe.query_async(&mut *conn).await?,
            Err(error) => {
                error!(error = ?error, "failed to cache guild channel");
            }
        }

        Ok(())
    }
}

#[async_trait]
impl UpdateCache for RoleCreate {
    const NAME: &'static str = "RoleCreate";

    async fn update(
        &self,
        redis: &RedisClient,
        _current_user: Id<ApplicationMarker>,
    ) -> Result<(), anyhow::Error> {
        let mut pipe = redis::pipe();

        super::resource::cache_role(&mut pipe, &self.role, self.guild_id)?;

        if let Some(mut guild) = redis.get::<CachedGuild>(&self.guild_id).await? {
            guild.roles.insert(self.role.id);
            pipe.set(guild.key(), guild.serialize_model()?);
        }

        let mut conn = redis.conn().await?;
        pipe.query_async(&mut *conn).await?;

        Ok(())
    }
}

#[async_trait]
impl UpdateCache for RoleDelete {
    const NAME: &'static str = "RoleDelete";

    async fn update(
        &self,
        redis: &RedisClient,
        _current_user: Id<ApplicationMarker>,
    ) -> Result<(), anyhow::Error> {
        let mut pipe = redis::pipe();

        if let Some(mut guild) = redis.get::<CachedGuild>(&self.guild_id).await? {
            guild.roles.remove(&self.role_id);
            pipe.set(guild.key(), guild.serialize_model()?);
        }

        pipe.del(CachedRole::key_from(&self.role_id));

        let mut conn = redis.conn().await?;
        pipe.query_async(&mut *conn).await?;

        Ok(())
    }
}

#[async_trait]
impl UpdateCache for RoleUpdate {
    const NAME: &'static str = "RoleUpdate";

    async fn update(
        &self,
        redis: &RedisClient,
        _current_user: Id<ApplicationMarker>,
    ) -> Result<(), anyhow::Error> {
        let mut pipe = redis::pipe();
        let mut conn = redis.conn().await?;

        super::resource::cache_role(&mut pipe, &self.role, self.guild_id)?;
        pipe.query_async(&mut *conn).await?;

        Ok(())
    }
}

#[async_trait]
impl UpdateCache for MemberAdd {
    const NAME: &'static str = "MemberAdd";

    async fn update(
        &self,
        redis: &RedisClient,
        current_user: Id<ApplicationMarker>,
    ) -> Result<(), anyhow::Error> {
        if self.user.id != current_user.cast() {
            // Only cache bot user
            return Ok(());
        }

        if let Some(mut guild) = redis.get::<CachedGuild>(&self.guild_id).await? {
            let cached = CurrentMember {
                id: self.user.id,
                communication_disabled_until: self.communication_disabled_until,
                roles: self.roles.iter().copied().collect(),
            };

            guild.current_member = Some(cached);
            redis.set(&guild).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl UpdateCache for MemberUpdate {
    const NAME: &'static str = "MemberUpdate";

    async fn update(
        &self,
        redis: &RedisClient,
        current_user: Id<ApplicationMarker>,
    ) -> Result<(), anyhow::Error> {
        if self.user.id != current_user.cast() {
            // Only cache bot user
            return Ok(());
        }

        if let Some(mut guild) = redis.get::<CachedGuild>(&self.guild_id).await? {
            let cached = CurrentMember {
                id: self.user.id,
                communication_disabled_until: self.communication_disabled_until,
                roles: self.roles.iter().copied().collect(),
            };

            guild.current_member = Some(cached);
            redis.set(&guild).await?;
        }
        Ok(())
    }
}
