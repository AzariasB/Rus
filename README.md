# Rus : a Rust URL Shortener

The project structure is largely inspired from the [sea orm actix example](https://github.com/SeaQL/sea-orm/tree/master/examples/actix_example)

Notable libs used :
- Actix for the web server
- Sea ORM to handle the database queries
- Tera (for now) to render html templates

## TODO
- [x] Validate url passed into parameters
- [ ] Handle in-memory caching of data
- [ ] Handle caching of data via redis
- [x] Return 404 when the url doesn't exit
- [ ] Handle link expiring (delete from database ?)
- [x] Save Ip address of user creating the url

## TODO v2
- [ ] Use Svelte as a frontend framework
- [ ] Add statistics to each newly created link
- [ ] Add account system to see the link stats
- [ ] An admin can delete all links
- [ ] A user can delete their link