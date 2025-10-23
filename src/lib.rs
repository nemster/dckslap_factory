use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
struct DckUserBadge {
    pub key_image_url: Url,
    #[mutable]
    pub has_dicks: bool,
    #[mutable]
    pub last_claim_time: Instant,
    #[mutable]
    pub claims: u32,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct DckUserBadgeMintEvent {
    id: u64,
    account: Global<Account>,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct DckslapClaimEvent {
    account: Global<Account>,
    claims_from_account: u32,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct GbofClaimEvent {
    account: Global<Account>,
    claims_from_account: u32,
}

#[blueprint]
#[events(
    DckUserBadgeMintEvent,
    DckslapClaimEvent,
    GbofClaimEvent,
)]
#[types(
    u64,
    Global<Account>,
)]
mod dckslap_factory {

    enable_method_auth! {
        roles {
            bot => updatable_by: [OWNER];
        },
        methods {
            mint_dckuserbadge => restrict_to: [bot];
            claim => PUBLIC;
            mint => restrict_to: [OWNER];
        }
    }

    struct DckslapFactory {
        dckslap_resource_manager: FungibleResourceManager,
        gbof_resource_manager: FungibleResourceManager,
        dckuserbadge_resource_manager: NonFungibleResourceManager,
        dckslap_per_claim: Decimal,
        claim_interval: i64,
        gbof_per_claim: Decimal,
        gbof_first_claim: u32,
        gbof_claim_increase: u32,
        gbof_claim_increase_increase: u32,
        next_dckuserbadge_id: u64,
        accounts: KeyValueStore<u64, Global<Account>>,
    }

    impl DckslapFactory {

        /* Instatiates a new DckslapFactory component.
         *
         * Input parameters:
         * - admin_badge_address: this resource address will be the owner of the component and
         * the resources
         * - bot_badge_address: a proof of this resource address will be needed to call the
         * mint_dckuserbadge method
         * - dckslap_initial_supply: the initial supply of DCKSLAP that will be returned by this
         * function
         * - gbof_initial_supply: the initial supply of GBOF that will be returned by this function
         * - dckslap_per_claim: how many DCKSLAP distribute at each successful claim
         * - claim_interval: interval in seconds between claims from the same account
         * - gbof_per_claim: how many GBOF distribute at each distribution
         * - gbof_first_claim: how many successful DCKSLAP claims are needed for the first GBOF
         * distribution
         * - gbof_claim_increase: fixed increase in claims for the next GBOF distribution
         * - gbof_claim_increase_increase: variable increase in claims for the next GBOF
         * distribution (this is multiplied by the number of distributions and summed to the
         * fixed increase)
         *
         * Outputs:
         * - the globalised DckslapFactory component
         * - a bucket of DCKSLAP
         * - a bucket of Great Ball Of Fire
         * - the resource address of the DckUserBadges that will be minted by mint_dckuserbadge
         */
        pub fn new(
            admin_badge_address: ResourceAddress,
            bot_badge_address: ResourceAddress,
            dckslap_initial_supply: Decimal,
            gbof_initial_supply: Decimal,
            dckslap_per_claim: Decimal,
            claim_interval: i64,
            gbof_per_claim: Decimal,
            gbof_first_claim: u32,
            gbof_claim_increase: u32,
            gbof_claim_increase_increase: u32,
        ) -> (
            Global<DckslapFactory>,
            FungibleBucket,
            FungibleBucket,
            ResourceAddress,
        ) {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(DckslapFactory::blueprint_id());

            let dckslap_bucket = ResourceBuilder::new_fungible(
                OwnerRole::Updatable(rule!(require(admin_badge_address)))
            )
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    roles {
                        metadata_setter => rule!(require(admin_badge_address));
                        metadata_setter_updater => rule!(require(admin_badge_address));
                        metadata_locker => rule!(require(admin_badge_address));
                        metadata_locker_updater => rule!(require(admin_badge_address));
                    },
                    init {
                        "name" => "DCKSLAP", updatable;
                        "symbol" => "DCKSLAP", updatable;
                    }
                })
                .mint_roles(mint_roles!(
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(require(admin_badge_address));
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(allow_all);
                    burner_updater => rule!(require(admin_badge_address));
                ))
                .mint_initial_supply(dckslap_initial_supply);
            let dckslap_resource_manager = FungibleResourceManager::from(
                dckslap_bucket.resource_address()
            );

            let gbof_bucket = ResourceBuilder::new_fungible(
                OwnerRole::Updatable(rule!(require(admin_badge_address)))
            )
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    roles {
                        metadata_setter => rule!(require(admin_badge_address));
                        metadata_setter_updater => rule!(require(admin_badge_address));
                        metadata_locker => rule!(require(admin_badge_address));
                        metadata_locker_updater => rule!(require(admin_badge_address));
                    },
                    init {
                        "name" => "Great Ball Of Fire", updatable;
                        "symbol" => "GBOF", updatable;
                    }
                })
                .mint_roles(mint_roles!(
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(require(admin_badge_address));
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(allow_all);
                    burner_updater => rule!(require(admin_badge_address));
                ))
                .mint_initial_supply(gbof_initial_supply);
            let gbof_resource_manager = FungibleResourceManager::from(
                gbof_bucket.resource_address()
            );

            let dckuserbadge_resource_manager = ResourceBuilder::new_integer_non_fungible::<DckUserBadge>(
                OwnerRole::Fixed(rule!(require(admin_badge_address)))
            )
                .metadata(metadata!(
                    roles {
                        metadata_setter => rule!(require(admin_badge_address));
                        metadata_setter_updater => rule!(require(admin_badge_address));
                        metadata_locker => rule!(require(admin_badge_address));
                        metadata_locker_updater => rule!(require(admin_badge_address));
                    },
                    init {
                        "name" => "Dck User Badge", updatable;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(require(admin_badge_address));
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(require(global_caller(component_address)));
                    burner_updater => rule!(require(admin_badge_address));
                ))
                .withdraw_roles(withdraw_roles!(
                    withdrawer => rule!(deny_all);
                    withdrawer_updater => rule!(require(admin_badge_address));
                ))
                .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                    non_fungible_data_updater => rule!(
                        require(global_caller(component_address))
                        || require(bot_badge_address)
                    );
                    non_fungible_data_updater_updater => rule!(require(admin_badge_address));
                ))
                .create_with_no_initial_supply();

            let dckslap_factory = Self {
                dckslap_resource_manager: dckslap_resource_manager,
                gbof_resource_manager: gbof_resource_manager,
                dckuserbadge_resource_manager: dckuserbadge_resource_manager,
                dckslap_per_claim: dckslap_per_claim,
                claim_interval: claim_interval,
                gbof_per_claim: gbof_per_claim,
                gbof_first_claim: gbof_first_claim,
                gbof_claim_increase: gbof_claim_increase,
                gbof_claim_increase_increase: gbof_claim_increase_increase,
                next_dckuserbadge_id: 1u64,
                accounts: KeyValueStore::new_with_registered_type(),
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::Updatable(rule!(require(admin_badge_address))))
                .roles(roles!(
                    bot => rule!(require(bot_badge_address));
                ))
                .with_address(address_reservation)
                .globalize();

            (
                dckslap_factory,
                dckslap_bucket,
                gbof_bucket,
                dckuserbadge_resource_manager.address(),
            )
        }

        /* Mints one or more DckUserBadges and sends them to the specified accounts
         *
         * You need the bot badge to invoke this method
         *
         * Input parameters:
         * - key_image_url: the image to set in the non fungible data of the DckUserBadges
         * - recipients: a vector of accounts that will receive one DckUserBadges each
         *
         * Events: a DckUserBadgeMintEvent for each successful distribution
         *
         * Note: unsuccessful distributions (because of accounts antispam settings) may cause a
         * hole in the sequence of DckUserBadges ids
         */
        pub fn mint_dckuserbadge(
            &mut self,
            key_image_url: Url,
            mut recipients: Vec<Global<Account>>,
        ) {
            let never = Instant::new(0i64);

            let mut dckuserbadge_sent = 0u64;

            for (i, account) in recipients.iter_mut().enumerate() {
                let id = self.next_dckuserbadge_id + i as u64;

                let dckuserbadge_bucket = self.dckuserbadge_resource_manager.mint_non_fungible(
                    &NonFungibleLocalId::integer(id),
                    DckUserBadge {
                        key_image_url: key_image_url.clone(),
                        has_dicks: true,
                        last_claim_time: never,
                        claims: 0u32,
                    }
                );

                let refund = account.try_deposit_or_refund(
                    dckuserbadge_bucket.into(),
                    None
                );

                match refund {
                    Some(bucket) => bucket.burn(),
                    None => {
                        Runtime::emit_event(
                           DckUserBadgeMintEvent {
                                id: id,
                                account: *account,
                            }
                        );

                        dckuserbadge_sent += 1u64;

                        self.accounts.insert(
                            id,
                            *account
                        );
                    }
                }
            }

            assert!(
                dckuserbadge_sent > 0,
                "No DckUserBadge sent"
            );

            self.next_dckuserbadge_id += recipients.len() as u64;
        }

        /* Claim DCKSLAP and eventually GBOF
         *
         * Input parameters:
         * - dckuserbadge_proof: a proof of ownership of a DckUserBadge
         *
         * Outputs:
         * - a bucket of DCKSLAP
         * - a bucket of GBOF or None
         *
         * Events:
         * - a DckslapClaimEvent
         * - eventually a GbofClaimEvent
         */
        pub fn claim(
            &mut self,
            dckuserbadge_proof: Proof,
        ) -> (
            FungibleBucket,
            Option<FungibleBucket>,
        ) {
            let non_fungible = dckuserbadge_proof.check_with_message(
                self.dckuserbadge_resource_manager.address(),
                "Incorrect proof",
            )
                .as_non_fungible()
                .non_fungible::<DckUserBadge>();

            let non_fungible_data = non_fungible.data();

            assert!(
                non_fungible_data.has_dicks,
                "You have no dick"
            );

            let now = Clock::current_time_rounded_to_seconds();
            assert!(
                non_fungible_data.last_claim_time.seconds_since_unix_epoch + self.claim_interval
                    <= now.seconds_since_unix_epoch,
                "Too soon"
            );

            self.dckuserbadge_resource_manager.update_non_fungible_data(
                &non_fungible.local_id(),
                "last_claim_time",
                now
            );

            let claims = non_fungible_data.claims + 1;
            self.dckuserbadge_resource_manager.update_non_fungible_data(
                &non_fungible.local_id(),
                "claims",
                claims
            );

            let dckslap_bucket = self.dckslap_resource_manager.mint(self.dckslap_per_claim);

            let id = match &non_fungible.local_id() {
                NonFungibleLocalId::Integer(local_id) => local_id.value(),
                _ => Runtime::panic("Incorrect proof".to_string()),
            };
            let account = self.accounts.get(&id).unwrap();

            Runtime::emit_event(
                DckslapClaimEvent {
                    account: *account,
                    claims_from_account: claims,
                }
            );

            let mut n = 1u32;
            let mut next_gbof_claim = self.gbof_first_claim;
            while claims > next_gbof_claim {
                next_gbof_claim += self.gbof_claim_increase + n * self.gbof_claim_increase_increase;
                n += 1;
            }
            let gbof_bucket = match claims == next_gbof_claim {
                true => {
                    Runtime::emit_event(
                        GbofClaimEvent {
                            account: *account,
                            claims_from_account: n,
                        }
                    );

                    Some(self.gbof_resource_manager.mint(self.gbof_per_claim))
                },

                false => None,
            };

            (
                dckslap_bucket,
                gbof_bucket,
            )
        }

        /* Mint DCKSLAP and GBOF
         *
         * You need the admin badge to invoke this method
         *
         * Input parameters:
         * - dckslap_amount: the amount of DCKSLAP to mint
         * - gbof_amount: the amount of GBOF to mint
         *
         * Outputs:
         * - a bucket of DCKSLAP
         * - a bucket of GBOF
         */
        pub fn mint(
            &mut self,
            dckslap_amount: Decimal,
            gbof_amount: Decimal,
        ) -> (FungibleBucket, FungibleBucket) {
            (
                self.dckslap_resource_manager.mint(dckslap_amount),
                self.gbof_resource_manager.mint(gbof_amount)
            )
        }
    }
}
