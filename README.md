# Rus : a Rust URL Shortener

The project structure is largely inspired from the [sea orm actix example](https://github.com/SeaQL/sea-orm/tree/master/examples/actix_example)

Notable libs used :
- Actix for the web server
- Sea ORM to handle the database queries
- Tera (for now) to render html templates

## Use with docker

To start rus with docker :
```bash
# This might take a while to build all the dependencies
docker build -t rus .
docker run --restart unless-stopped --env RUS_HOST=0.0.0.0 --env RUS_PORT=8000 --env RUS_DATABASE_URL="postgresql://<user>:<password>@127.0.0.1/<databasename>"  -p "8000:8000" rus 
```
Then go with a web browser to the address localhost:8000 to see if everything's going well.

## TODO
- [x] Add Docker support
- [x] Add Github pipeline
- [x] Validate url passed into parameters
- [x] Handle in-memory caching of data
- [x] Handle caching of data via redis
- [x] Return 404 when the url doesn't exit
- [x] Save Ip address of user creating the url
- [x] Update last access date when the link is clicked
- [x] Use the correct logging system
- [ ] Handle link expiring (delete from database ?)

## TODO v2
- [ ] Use Svelte as a frontend framework
- [ ] Add statistics to each newly created link
- [ ] Add account system to see the link stats
- [ ] An admin can delete all links
- [ ] A user can delete their link