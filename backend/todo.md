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

* cmd `/rst add {name} {address}`
* cmd `/rst search {fuzzy-pattern}`
    - return list of id-matches pair
* cmd `/rst edit {id}`
    - return menus
        * [update]
        * [new dishes]
        * [list dishes]
        * [delete]

## Frontend
