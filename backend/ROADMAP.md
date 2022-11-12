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

## TG Bot

- Commands
    - [ ] `/rst add/search/edit` **[CURRENT]**
    - [ ] `/meal add/upd`
    - [ ] `/review add/upd`
- [ ] Update dispatcher

---

- [x] cmd `/rst add {name} {address}`
- [x] cmd `/rst search {fuzzy-pattern}`
    - [x] return list of id-matches pair
- [x] cmd `/rst edit {id}`
    - return menus
        * [update]
        * [new dishes]
        * [list dishes]
        * [delete]
- [x] Callback dispatcher
- [x] Setup dialogue
- [x] Update callback
    - [x] edit message
    - [x] update menu => `[update name | update address | cancel(Go back)]`
- [x] New dishes
    - [x] new dialogue
    - [x] receive name
    - [x] receive image (Optional)
    - [x] update database
- [x] List dishes: new message => query dishes for current restaurant
- [ ] Delete: edit message => update menu => [Confirm | Cancel]

## Frontend
