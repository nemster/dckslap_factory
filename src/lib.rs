use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
struct DckUserBadge {
    #[mutable]
    pub key_image_url: Url,
    #[mutable]
    pub has_dicks: bool,
    #[mutable]
    pub last_claim_time: Instant,
}

#[derive(ScryptoSbor, Clone)]
struct User {
    claims: u32,
    claims_current_group: u32,
    burned_dckslap: u32,
}

#[blueprint]
#[types(
    u64,
    User,
)]
mod spank_bank {

    enable_method_auth! {
        roles {
            bot => updatable_by: [OWNER];
        },
        methods {
            claim => PUBLIC;
            burn => PUBLIC;
            pay_claim => PUBLIC;
            mint => restrict_to: [OWNER];
            withdraw_reddicks => restrict_to: [OWNER];
            deposit_xrd => PUBLIC;
            update_settings => restrict_to: [OWNER];
        }
    }

    struct SpankBank {
        dckslap_resource_manager: FungibleResourceManager,
        gbof_resource_manager: FungibleResourceManager,
        dckuserbadge_resource_manager: NonFungibleResourceManager,
        dckslap_per_claim: Decimal,
        claims_per_group: u32,
        claim_group_interval: i64,
        gbof_per_claim: Decimal,
        gbof_first_claim: u32,
        gbof_claim_increase: u32,
        gbof_claim_increase_increase: u32,
        dckslap_per_gbof: u32,
        reddicks_vault: FungibleVault,
        reddicks_per_claim: u32,
        users: KeyValueStore<u64, User>,
        xrd_vault: FungibleVault,
    }

    impl SpankBank {

        /* Instatiates a new SpankBank component and changes permissione on existing resources so
         * that it can manage them.
         *
         * Input parameters:
         * - admin_badge_bucket: the badge to manage the existing resources
         * - bot_badge_address: resource address of the bot badge
         * - dckuserbadge_address: resource address of the Dck User Badge
         * - dckslap_address: resource address of DCKSLAP
         * - gbof_address: resource address of GBOF
         * - dckslap_per_claim: how many DCKSLAP distribute at each successful claim
         * - claims_per_group: how many claims can an account do in a period of time
         * - claim_group_interval: interval in seconds between claim groups from the same account
         * - gbof_per_claim: how many GBOF distribute at each distribution
         * - gbof_first_claim: how many successful DCKSLAP claims are needed for the first GBOF
         * distribution
         * - gbof_claim_increase: fixed increase in claims for the next GBOF distribution
         * - gbof_claim_increase_increase: variable increase in claims for the next GBOF
         * distribution (this is multiplied by the number of distributions and summed to the
         * fixed increase)
         * - dckslap_per_gbof: how many DCKSLAP can be burned to get a GBOF
         * - reddicks_address: the resource address of the REDDICKS coin
         * - reddicks_per_claim: how many REDDICKS a user has to pay for an additional claim
         *
         * Outputs:
         * - the globalised SpankBank component
         * - the privided admin badge bucket
         */
        pub fn new(
            admin_badge_bucket: FungibleBucket,
            bot_badge_address: ResourceAddress,
            dckuserbadge_address: ResourceAddress,
            dckslap_address: ResourceAddress,
            gbof_address: ResourceAddress,
            dckslap_per_claim: Decimal,
            claims_per_group: u32,
            claim_group_interval: i64,
            gbof_per_claim: Decimal,
            gbof_first_claim: u32,
            gbof_claim_increase: u32,
            gbof_claim_increase_increase: u32,
            dckslap_per_gbof: u32,
            reddicks_address: ResourceAddress,
            reddicks_per_claim: u32,
        ) -> (
            Global<SpankBank>,
            FungibleBucket
        ) {
            let admin_badge_address = admin_badge_bucket.resource_address();

            let (address_reservation, component_address) =
                Runtime::allocate_component_address(SpankBank::blueprint_id());

            let dckuserbadge_resource_manager = NonFungibleResourceManager::from(dckuserbadge_address);

            admin_badge_bucket.authorize_with_amount(
                1,
                || dckuserbadge_resource_manager.set_updatable_non_fungible_data(
                    rule!(
                        require(global_caller(component_address))
                        || require(bot_badge_address)
                    )
                )
            );

            let dckslap_resource_manager = FungibleResourceManager::from(dckslap_address);
           
            admin_badge_bucket.authorize_with_amount(
                1,
                || {
                    dckslap_resource_manager.set_mintable(
                        rule!(
                            require(global_caller(component_address))
                        )
                    );
                    dckslap_resource_manager.set_burnable(
                        rule!(
                            require(global_caller(component_address))
                        )
                    );
                }
            );

            let gbof_resource_manager = FungibleResourceManager::from(gbof_address);
            
            admin_badge_bucket.authorize_with_amount(
                1,
                || gbof_resource_manager.set_mintable(
                    rule!(
                        require(global_caller(component_address))
                    )
                )
            );

            let spank_bank = Self {
                dckslap_resource_manager: dckslap_resource_manager,
                gbof_resource_manager: gbof_resource_manager,
                dckuserbadge_resource_manager: dckuserbadge_resource_manager,
                dckslap_per_claim: dckslap_per_claim,
                claims_per_group: claims_per_group,
                claim_group_interval: claim_group_interval,
                gbof_per_claim: gbof_per_claim,
                gbof_first_claim: gbof_first_claim,
                gbof_claim_increase: gbof_claim_increase,
                gbof_claim_increase_increase: gbof_claim_increase_increase,
                dckslap_per_gbof: dckslap_per_gbof,
                reddicks_vault: FungibleVault::new(reddicks_address),
                reddicks_per_claim: reddicks_per_claim,
                users: KeyValueStore::new_with_registered_type(),
                xrd_vault: FungibleVault::new(XRD),
            }
                .instantiate()
                .prepare_to_globalize(OwnerRole::Updatable(rule!(require(admin_badge_address))))
                .with_address(address_reservation)
                .globalize();

            (
                spank_bank,
                admin_badge_bucket,
            )
        }

        /* Internal method to check a user badge proof and return all of the informations
         * associated with the user
         *
         * Input parameters:
         * - dckuserbadge_proof: a user badge proof
         *
         * Output parameters:
         * - the NonFungibleData of the user badge
         * - the NonFungibleLocalId of the user badge
         * - the numeric id extracted from the NonFungibleLocalId
         * - internal information about the user stored in the users KVS
         */
        fn check_user_badge(
            &mut self,
            dckuserbadge_proof: Proof,
        ) -> (
            DckUserBadge,
            NonFungibleLocalId,
            u64,
            User
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

            let local_id = non_fungible.local_id().clone();

            let id = match &local_id {
                NonFungibleLocalId::Integer(local_id) => local_id.value(),
                _ => Runtime::panic("Should not happen".to_string()),
            };

            let user = match self.users.get(&id) {
                Some(user) => user.clone(),

                None => {
                    let user = User {
                        claims: 0,
                        claims_current_group: 0,
                        burned_dckslap: 0,
                    };

                    self.users.insert(id, user.clone());

                    user
                },
            };

            (
                non_fungible_data,
                local_id,
                id,
                user,
            )
        }

        /* Internal method to pay the fees of the curent transaction
         *
         * Input parameters:
         * - amount: the maximum amount of XRD to pay
         */
        fn pay_fees(
            &mut self,
            amount: Decimal,
        ) {
            if self.xrd_vault.amount() >= amount {
                self.xrd_vault.lock_contingent_fee(amount);
            }
        }

        /* Internal method to handle both free and paid claims
         *
         * Input parameters:
         * - id: numeric identifier of user's badge
         * - user: details about the user claiming
         *
         * Outputs:
         * - a bucket of DCKSLAP
         * - a bucket of GBOF or None
         */
        fn do_claim(
            &mut self,
            id: u64,
            mut user: User,
        ) -> (FungibleBucket, Option<FungibleBucket>) {
            user.claims += 1;

            let dckslap_bucket = self.dckslap_resource_manager.mint(self.dckslap_per_claim);

            let mut n = 1u32;
            let mut next_gbof_claim = self.gbof_first_claim;
            while user.claims > next_gbof_claim {
                next_gbof_claim += self.gbof_claim_increase + n * self.gbof_claim_increase_increase;
                n += 1;
            }
            let gbof_bucket = match user.claims == next_gbof_claim {
                true => Some(self.gbof_resource_manager.mint(self.gbof_per_claim)),

                false => None,
            };

            self.users.insert(id, user);

            (
                dckslap_bucket,
                gbof_bucket,
            )
        }

        /* Claim DCKSLAP and eventually GBOF
         *
         * This method updates the user struct but doesn't save it in the users KVS, the do_claim
         * method will take care of this
         *
         * Input parameters:
         * - dckuserbadge_proof: a proof of ownership of a DckUserBadge
         *
         * Outputs:
         * - a bucket of DCKSLAP
         * - a bucket of GBOF or None
         */
        pub fn claim(
            &mut self,
            dckuserbadge_proof: Proof,
        ) -> (
            FungibleBucket,
            Option<FungibleBucket>,
        ) {
            self.pay_fees(dec![1]);

            let (non_fungible_data, local_id, id, mut user) = self.check_user_badge(dckuserbadge_proof);

            let now = Clock::current_time_rounded_to_seconds();


            if non_fungible_data.last_claim_time.seconds_since_unix_epoch
                + self.claim_group_interval > now.seconds_since_unix_epoch {

                assert!(
                    user.claims_current_group < self.claims_per_group,
                    "No more free claims today"
                );


                user.claims_current_group += 1;

            } else {

                self.dckuserbadge_resource_manager.update_non_fungible_data(
                    &local_id,
                    "last_claim_time",
                    now
                );

                user.claims_current_group = 1;
            }

            self.do_claim(id, user)
        }

        /* Burn DCKSLAP and eventually obtain GBOF
         *
         * Input parameters:
         * - dckuserbadge_proof: a proof of ownership of a DckUserBadge
         * - dckslap_bucket: a bucket containing at least one DCKSLAP
         *
         * Outputs:
         * - a bucket of GBOF or None
         * - a bucket containing eventual excess DCKSLAP
         */
        pub fn burn(
            &mut self,
            dckuserbadge_proof: Proof,
            mut dckslap_bucket: FungibleBucket,
        ) -> (Option<FungibleBucket>, FungibleBucket) {
            self.pay_fees(dec![1]);

            assert!(
                dckslap_bucket.resource_address() == self.dckslap_resource_manager.address(),
                "Wrong coin"
            );

            let (_, _, id, mut user) = self.check_user_badge(dckuserbadge_proof);

            dckslap_bucket.take(1).burn();

            user.burned_dckslap += 1;

            let gbof_bucket = match user.burned_dckslap >= self.dckslap_per_gbof {
                true => {
                    user.burned_dckslap = 0;

                    Some(self.gbof_resource_manager.mint(dec![1]))
                },
                false => None,
            };

            self.users.insert(id, user);

            (
                gbof_bucket,
                dckslap_bucket,
            )
        }

        /* Claim DCKSLAP paying with REDDICKS
         *
         * Input parameters:
         * - dckuserbadge_proof: a proof of ownership of a DckUserBadge
         * - reddicks_bucket: a bucket of REDDICKS
         *
         * Outputs:
         * - a bucket of DCKSLAP
         * - a bucket of GBOF or None
         * - a bucket of eventual excess REDDICKS
         */
        pub fn pay_claim(
            &mut self,
            dckuserbadge_proof: Proof,
            mut reddicks_bucket: FungibleBucket,
        ) -> (FungibleBucket, Option<FungibleBucket>, FungibleBucket) {
            self.pay_fees(dec![1]);

            let (_, _, id, user) = self.check_user_badge(dckuserbadge_proof);

            self.reddicks_vault.put(
                reddicks_bucket.take(
                    self.reddicks_per_claim
                )
            );

            let (dckslap_bucket, gbof_bucket) = self.do_claim(id, user);

            (
                dckslap_bucket,
                gbof_bucket,
                reddicks_bucket,
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

        /* Withdraw REDDICKS paid by users
         *
         * You need the admin badge to invoke this method
         *
         * Outputs:
         * - a bucket of REDDICKS
         */
        pub fn withdraw_reddicks(&mut self) -> FungibleBucket {
            self.reddicks_vault.take_all()
        }

        /* Deposit XRDs to pay users' transactions
         *
         * Input parameters:
         * - xrd_bucket: a bucket of XRD
         */
        pub fn deposit_xrd(
            &mut self,
            xrd_bucket: FungibleBucket,
        ) {
            self.xrd_vault.put(xrd_bucket);
        }

        /* Update component settings
         *
         * You need the admin badge to invoke this method
         *
         * Input parameters:
         * - dckslap_per_claim: how many DCKSLAP distribute at each successful claim
         * - claims_per_group: how many claims can an account do in a period of time
         * - claim_group_interval: interval in seconds between claim groups from the same account
         * - gbof_per_claim: how many GBOF distribute at each distribution
         * - gbof_first_claim: how many successful DCKSLAP claims are needed for the first GBOF
         * distribution
         * - gbof_claim_increase: fixed increase in claims for the next GBOF distribution
         * - gbof_claim_increase_increase: variable increase in claims for the next GBOF
         * distribution (this is multiplied by the number of distributions and summed to the
         * fixed increase)
         * - dckslap_per_gbof: how many DCKSLAP can be burned to get a GBOF
         * - reddicks_per_claim: how many REDDICKS a user has to pay for an additional claim
         */
        pub fn update_settings(
            &mut self,
            dckslap_per_claim: Decimal,
            claims_per_group: u32,
            claim_group_interval: i64,
            gbof_per_claim: Decimal,
            gbof_first_claim: u32,
            gbof_claim_increase: u32,
            gbof_claim_increase_increase: u32,
            dckslap_per_gbof: u32,
            reddicks_per_claim: u32,
        ) {
            self.dckslap_per_claim = dckslap_per_claim;
            self.claims_per_group = claims_per_group;
            self.claim_group_interval = claim_group_interval;
            self.gbof_per_claim = gbof_per_claim;
            self.gbof_first_claim = gbof_first_claim;
            self.gbof_claim_increase = gbof_claim_increase;
            self.gbof_claim_increase_increase = gbof_claim_increase_increase;
            self.dckslap_per_gbof = dckslap_per_gbof;
            self.reddicks_per_claim = reddicks_per_claim;
        }
    }
}
