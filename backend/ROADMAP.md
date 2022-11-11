## Structure

- [ ] Restaurant
    - [ ] Alias
    - [ ] Name
    - [ ] Address
    - [ ] Meal
        - [ ] Name
        - [ ] Alias
        - [ ] Image (Optional)
        - [ ] Review
            - [ ] Who
            - [ ] Details
            - [ ] Score

## Backend (The TG Bot)

- Commands
    - [ ] `/meal add/upd`
    - [ ] `/review add/upd`
- Database

### Restaurant Manage

* 2022-11-10
    * cmd `/rst add {name} {address}`
    * cmd `/rst search {fuzzy-pattern}`
        - return list of id-matches pair
    * cmd `/rst edit {id}`
        - return menus
            * [update]
            * [new dishes]
            * [list dishes]
            * [delete]
* 2022-11-12
    * Callback handler
    * Setup dialogue
    * Update callback: edit message => update menu => [update name | update address | cancel(Go back)]
    * New dishes: new dialogue => name => image (Optional)
    * List dishes: new message => query dishes for current restaurant
    * Delete: edit message => update menu => [Confirm | Cancel]

## Frontend
