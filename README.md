# DCKSLAP Factory

DckslapFactory is a blueprint to manage the distribution of two fungibles (`DCKSLAP` and `GBOF`) and a non fungible (`Dck User Badge`) that is needed to keep track of users' claims.  
`DCKSLAP` can be claimed periodically by the users who own the non fungible; the claim operation eventually returns some `GBOF` too.  
The `claim_interval` set in the `new` method determines how often a user can do a claim.  

The `<GBOF_FIRST_CLAIM>`, `<GBOF_CLAIM_INCREASE>` and `<GBOF_CLAIM_INCREASE_INCREASE>` parameters determine whether the claim returns `GBOF` too.  
As an example setting `<GBOF_FIRST_CLAIM>`=69, `<GBOF_CLAIM_INCREASE>`=69 and `<GBOF_CLAIM_INCREASE_INCREASE>`=28, will make so that only the claims number 69, 69+69+28=166, 166+69+28+28=291, 291+69+28+28+28=444, ... will return `GBOF` (quadratic backoff).  

Offchain script are needed to tell the component who to send the `Dck User Badge` to and to enable/disable them (`has_dicks` non fungible data).  
The same `bot badge` is needed to update `Dck User Bagde` non fungible data and invoke the `mint_dckuserbadge` method.  

A user can also pay with `REDDICKS` for an additonal `DCKSLAP` claim; the parameter `<REDDICKS_PER_CLAIM>` is the price to pay.  
Paid claims do not interfere with the time limited claims.  

It is also possible to burn `DCKSLAP` (one at a time); when the user has burned enough `DCKSLAP` (`dckslap_per_gbof` parameter) a `GBOF` claim happens.  

## `new`
Use this function to instatiate a new DckslapFactory component and mint an initial supply of both `DCKSLAP` and `GBOF`.  

```
CALL_FUNCTION
    Address("<PACKAGE_ADDRESS>")
    "DckslapFactory"
    "new"
    Address("<ADMIN_BADGE_ADDRESS>")
    Address("<BOT_BADGE_ADDRESS>")
    Decimal("<DCKSLAP_INITIAL_SUPPLY>")
    Decimal("<GBOF_INITIAL_SUPPLY>")
    Decimal("<DCKSLAP_PER_CLAIM>")
    <CLAIM_INTERVAL>i64
    Decimal("<GBOF_PER_CLAIM>")
    <GBOF_FIRST_CLAIM>u32
    <GBOF_CLAIM_INCREASE>u32
    <GBOF_CLAIM_INCREASE_INCREASE>u32
    <DCKSLAP_PER_GBOF>u32
    Address("<REDDICKS_ADDRESS>")
    <REDDICKS_PER_CLAIM>u32
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<PACKAGE_ADDRESS>`: the address of the package containing the `DckslapFactory` blueprint.  
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
`<DCKSLAP_PER_GBOF>`: the number of `DCKSLAP` a user can burn to get a `GBOF`
`<REDDICKS_ADDRESS>`: the resource address of the `REDDICKS` coin  
`<REDDICKS_PER_CLAIM>`: how many `REDDICKS` a user has to pay for an additional claim  
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

## `burn`
A user can invoke this method to burn a `DCKSLAP` and eventually obtain a `GBOF`

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "withdraw"
    Address("<DCKSLAP_ADDRESS>")
    Decimal("1")
;
TAKE_ALL_FROM_WORKTOP
    Address("<DCKSLAP_ADDRESS>")
    Bucket("dckslap")
;
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
    "burn"
    Proof("dckuserbadge_proof")
    Bucket("dckslap")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>`: address of the user account.  
`<DCKSLAP_ADDRESS>`: resource address of `DCKSLAP` coins.  
`<DCKUSERBADGE_ADDRESS>`: resource address of the `Dck User Badge`.  
`<DCKUSERBADGE_ID>`: numeric id of the `Dck User Badge` in the user's account.  
`<COMPONENT_ADDRESS>`: the component created by the `new` function.  

A `GBOF` is returned when the user has burned `<DCKSLAP_PER_GBOF>` `DCKSLAP`; in this case a `GbofClaimEvent` event is emitted. The event contains user's account address and the number of times this account has received `GBOFs`.  
If the user burns multiple `DCKSLAP` in a single operation those will be counted as just one; so he really needs to invoke this method `<DCKSLAP_PER_GBOF>` times.  

## `pay_claim`
A user can call this method to pay for an unscheduled `DCKSLAP` claim paying with `REDDICKS`  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "withdraw"
    Address("<REDDICKS_ADDRESS>")
    Decimal("1")
;
TAKE_ALL_FROM_WORKTOP
    Address("<REDDICKS_ADDRESS>")
    Bucket("reddicks")
;
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
    "pay_claim"
    Proof("dckuserbadge_proof")
    Bucket("reddicks")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

`<ACCOUNT_ADDRESS>`: address of the user account.  
`<REDDICKS_ADDRESS>`: resource address of `REDDICKS` coins.  
`<DCKUSERBADGE_ADDRESS>`: resource address of the `Dck User Badge`.  
`<DCKUSERBADGE_ID>`: numeric id of the `Dck User Badge` in the user's account.  
`<COMPONENT_ADDRESS>`: the component created by the `new` function.  

A `DckslapClaimEvent` event is emitted. The event contains user's account address and the number of times this account has received `DCKSLAPs`.  

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

## `withdraw_reddicks`
The admin can invoke this method to withdraw all of the `REDDICKS` paid by the users.  

```
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_amount"
    Address("<ADMIN_BADGE_ADDRESS>")
    Decimal("1")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "withdraw_reddicks"
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

