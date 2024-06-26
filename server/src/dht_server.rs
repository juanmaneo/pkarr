use std::{
    fmt::Debug,
    net::{SocketAddr, ToSocketAddrs},
};

use pkarr::{
    mainline::{
        self,
        rpc::{
            messages::{
                GetMutableResponseArguments, GetValueRequestArguments, RequestSpecific,
                RequestTypeSpecific, ResponseSpecific,
            },
            Rpc,
        },
        server::Server,
        MutableItem,
    },
    PkarrCache,
};

use tracing::debug;

use crate::{cache::HeedPkarrCache, rate_limiting::IpRateLimiter};

/// DhtServer with Rate limiting
pub struct DhtServer {
    inner: mainline::server::DhtServer,
    resolvers: Option<Vec<SocketAddr>>,
    cache: Box<crate::cache::HeedPkarrCache>,
    minimum_ttl: u32,
    maximum_ttl: u32,
    rate_limiter: IpRateLimiter,
}

impl Debug for DhtServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Resolver")
    }
}

impl DhtServer {
    pub fn new(
        cache: Box<HeedPkarrCache>,
        resolvers: Option<Vec<String>>,
        minimum_ttl: u32,
        maximum_ttl: u32,
        rate_limiter: IpRateLimiter,
    ) -> Self {
        Self {
            // Default DhtServer used to stay a good citizen servicing the Dht.
            inner: mainline::server::DhtServer::default(),
            cache,
            resolvers: resolvers.map(|resolvers| {
                resolvers
                    .iter()
                    .flat_map(|resolver| resolver.to_socket_addrs())
                    .flatten()
                    .collect::<Vec<_>>()
            }),
            minimum_ttl,
            maximum_ttl,
            rate_limiter,
        }
    }
}

impl Server for DhtServer {
    fn handle_request(
        &mut self,
        rpc: &mut Rpc,
        from: SocketAddr,
        transaction_id: u16,
        request: &RequestSpecific,
    ) {
        if let RequestSpecific {
            request_type: RequestTypeSpecific::GetValue(GetValueRequestArguments { target, .. }),
            ..
        } = request
        {
            let should_query = if let Some(cached) = self.cache.get(target) {
                debug!(
                    public_key = ?cached.public_key(),
                    ?target,
                    "cache hit responding with packet!"
                );

                // Respond with what we have, even if expired.
                let mutable_item = MutableItem::from(&cached);

                rpc.response(
                    from,
                    transaction_id,
                    ResponseSpecific::GetMutable(GetMutableResponseArguments {
                        responder_id: *rpc.id(),
                        // Token doesn't matter much, as we are most likely _not_ the
                        // closest nodes, so we shouldn't expect an PUT requests based on
                        // this response.
                        token: vec![0, 0, 0, 0],
                        nodes: None,
                        v: mutable_item.value().to_vec(),
                        k: mutable_item.key().to_vec(),
                        seq: *mutable_item.seq(),
                        sig: mutable_item.signature().to_vec(),
                    }),
                );

                // If expired, we try to hydrate the packet from the DHT.
                let expires_in = cached.expires_in(self.minimum_ttl, self.maximum_ttl);
                let expired = expires_in == 0;

                if expired {
                    debug!(
                        public_key = ?cached.public_key(),
                        ?target,
                        ?expires_in,
                        "cache expired, querying the DHT to hydrate our cache for later."
                    );
                };

                expired
            } else {
                debug!(
                    ?target,
                    "cache miss, querying the DHT to hydrate our cache for later."
                );
                true
            };

            //  Either cache miss or expired cached packet
            if should_query {
                // Rate limit nodes that are making too many request forcing us to making too
                // many queries, either by querying the same non-existent key, or many unique keys.
                if self.rate_limiter.is_limited(&from.ip()) {
                    debug!(?from, "Resolver rate limiting");
                } else {
                    rpc.get(
                        *target,
                        RequestTypeSpecific::GetValue(GetValueRequestArguments {
                            target: *target,
                            seq: None,
                            salt: None,
                        }),
                        None,
                        self.resolvers.to_owned(),
                    );
                };
            }
        };

        // Do normal Dht request handling (peers, mutable, immutable, and routing).
        self.inner
            .handle_request(rpc, from, transaction_id, request)
    }
}
