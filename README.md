# DCKSLAP Factory

DckslapFactory is a blueprint to manage the distribution of two fungibles (`DCKSLAP` and `GBOF`) and a non fungible (`Dck User Badge`) that is needed to keep track of users' claims.  
`DCKSLAP` can be claimed periodically by the users who own the non fungible; the claim operation eventually returns some `GBOF` too.  
The `claim_interval` set in the `new` method determines how often a user can do a claim.  

The `gbof_first_claim`, `gbof_claim_increase` and `gbof_claim_increase_increase` parameters determine whether the claim returns `GBOF` too.  
As an example setting `gbof_first_claim`=15, `gbof_claim_increase`=20 and `gbof_claim_increase_increase`=10, will make so that only the claims number 15, 15+20+10=45, 45+20+10+10=85, 85+20+10+10+10=135, ... will return `GBOF` (quadratic backoff).  

Offchain script are needed to tell the component who to send the `Dck User Badge` to and to enable/disable them (`has_dicks` non fungible data).  
The same `bot badge` is needed to update `Dck User Bagde` non fungible data and invoke the `mint_dckuserbadge` method.  

## `new`
Use this function to instatiate a new DckslapFactory component and mint an initial supply of both `DCKSLAP` and `GBOF`.  

```
CALL_FUNCTION
    Address("<BLUEPRINT_ADDRESS>")
    "DckslapFactory"
    "new"
    Address("<ADMIN_BADGE_ADDRESS>")
    Address("<BOT_BADGE_ADDRESS>")
    Decimal("<DCKSLAP_INITIAL_SUPPLY>")
    Decimal("<GBOF_INITIAL_SUPPLY>")
    Decimal("<DCKSLAP_PER_CLAIM>")
    <CLAIM_INTERVAL>i64
    Decimal("<GBOF_PER_CLAIM>")
    <GBOF_FIRST_CLAIM>u64
    <GBOF_CLAIM_INCREASE>u64
    <GBOF_CLAIM_INCREASE_INCREASE>u64
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<BLUEPRINT_ADDRESS>`: the address of the `DckslapFactory` blueprint.  
`<ADMIN_BADGE_ADDRESS>`: this resource address will be the owner of the component and the resources.  
`<BOT_BADGE_ADDRESS>`: a proof of this resource address will be needed to call the `mint_dckuserbadge` method.  
`<DCKSLAP_INITIAL_SUPPLY>`: the initial supply of `DCKSLAP` that will be returned by this function.  
`<GBOF_INITIAL_SUPPLY>`: the initial supply of `GBOF` that will be returned by this function.  
`<DCKSLAP_PER_CLAIM>`: how many `DCKSLAP` distribute at each successful claim.  
`<CLAIM_INTERVAL>`: interval in seconds between claims from the same account.  
`<GBOF_PER_CLAIM>`: how many `GBOF` distribute at each distribution.  
`<GBOF_FIRST_CLAIM>`: how many successful `DCKSLAP` claims are needed for the first GBOF distribution.  
`<GBOF_CLAIM_INCREASE>`: fixed increase in claims for the next `GBOF` distribution.  
`<GBOF_CLAIM_INCREASE_INCREASE>`: variable increase in claims for the next `GBOF` distribution (this is multiplied by the number of distributions and summed to the fixed increase).  
`<ACCOUNT_ADDRESS>`: the account to deposit the initial supply in.  

## `mint_dckuserbadge`
Invoke this method to mint one or more `Dck User Badge` and send them to the specified account(s)  

```
CALL_METHOD
    Address("<BOT_ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<BOT_BADGE_ADDRESS>")
    Decimal("1")
; 
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "mint_dckuserbadge"
    "<IMAGE_URL>"
    Array<Address>(
        Address("<ACCOUNT_ADDRESS>")
    )
;
```

`<BOT_ACCOUNT_ADDRESS>`: address of the account used by the bot.  
`<BOT_BADGE_ADDRESS>`: resource address of the bot badge specified when calling the `new` function.  
`<COMPONENT_ADDRESS>`: the component created by the `new` function.  
`<IMAGE_URL>`: the URL of the image to use as `key_image_url` in the NFT.  
`<ACCOUNT_ADDRESS>`: the address of the account to send the `Dck User Badge` to.  

This method emits a `DckUserBadgeMintEvent` event for each recipient.  
The event contains the account address and the unique numeric id of the sent `Dck User Badge`.  

If the recipient account has antispam settings that prevent this method to send it the `Dck User Badge`, then the minted `Dck User Badge` is burned. This may cause a "hole" in the id sequence if in the same call there are successful and unsuccessful recipients.  

Only if no recipent is successful, the transaction fails.  

## `claim`
A user can call this method to get some `DCKSLAP` and eventually some `GBOF` too.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<DCKUSERBADGE_ADDRESS>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<DCKUSERBADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("dckuserbadge_proof")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "claim"
    Proof("dckuserbadge_proof")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>`: address of the user account.  
`<DCKUSERBADGE_ADDRESS>`: resource address of the `Dck User Badge`.  
`<DCKUSERBADGE_ID>`: numeric id of the `Dck User Badge` in the user's account.  
`<COMPONENT_ADDRESS>`: the component created by the `new` function.  

This method fails if less than `claim_interval` seconds has passed since the last claim from this user or if the `has_dicks` non fungible data in the `Dck User Badge` is false.  

Upon success the `claim` method emits a `DckslapClaimEvent` event specifying the account address and the number of claims from this account.  
If `GBOFs` are returned too, this method will emit a `GbofClaimEvent` event too specifying the account address and the number of times this account has received `GBOFs`.  

## `mint`
The admin can invoke this method to mint new `DCKSLAP` and/or `GBOF`.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<ADMIN_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "mint"
    Decimal("<DCKSLAP_AMOUNT>")
    Decimal("<GBOF_AMOUNT>")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>`: address of the admin account.  
`<ADMIN_BADGE_ADDRESS>`: resource address of the admin badge.  
`<COMPONENT_ADDRESS>`: the component created by the `new` function.  
`<DCKSLAP_AMOUNT>`: the amount of `DCKSLAP` to mint.  
`<GBOF_AMOUNT>`: the amount of `GBOF` to mint.  

