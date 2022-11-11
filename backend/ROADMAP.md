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
- [ ] Callback dispatcher
- [ ] Setup dialogue
- [ ] Update callback
    - [ ] edit message
    - [ ] update menu => `[update name | update address | cancel(Go back)]`
- [ ] New dishes
    - [ ] new dialogue
    - [ ] receive name
    - [ ] receive image (Optional)
    - [ ] update database
- [ ] List dishes: new message => query dishes for current restaurant
- [ ] Delete: edit message => update menu => [Confirm | Cancel]

## Frontend
