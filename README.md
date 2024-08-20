# acid4sigmas-backend
This is the backend of acid4sigmas.systems! 

# How to use this backend

## prerequisites
- [rust language](https://www.rust-lang.org/tools/install)
- [postgre sql])(https://www.postgresql.org/)
- a domain (you can buy one [here](https://www.namecheap.com/)
- have your domain set up probably so that it is reachable to the internet
- an brevo account or any other smtp provider 

## Secret.toml

a Secret.toml should look like this

SECRET_KEY="your_secret_key_for_jsonwebtoken"
DB_NAME="your_postgre_db_name"
DB_PW="your_db_password"
DB_PORT="5432" # port of your db connection

# email 
NO_REPLY_EMAIL="your-no-reply-email@yourservicedomain.com" 
SMTP_USERNAME="your_smtp_username"
SMTP_PASSWORD="your_smtp_password"
SMTP_RELAY="your_smtp_password"


### Secret_key
the secret_key is used to create a jsonwebtoken which will be returned to the user if they create a new account | login into their existing account

### Db Name
the db name is basically the name of your database, if you are not familiar with how to create a new database with postgreSQL i would recommend using the default database provided by PostgreSQL which is `postgres`

### Db passowrd
the db password is used to authenticate yourself with postgreSQL

### Db port
the db port is used to establish the connection to your database

### also important about the db section
at the moment only localhost postgreSQL dbs are supported, this might change in the future. depending on the needs of this project

### No reply email
this is the email address which the backend will use to send for example email verifiactions to the users email address

### SMTP username & password & relay
The smtp username and smtp password is required for the backend to authenticate with your email provider.
the smtp_relay is used for the backend to connec to the smtp server

anyways. if you are more interested about how to set up your domain with smtp using cloudflare and brevo (or others)
here is a [youtube video](https://www.youtube.com/watch?v=nNGcvz1Sc_8)

## Static html hosting
I also made an implementation to host static html files on the root index of the website
to host your html just place your `index.html` in the `static` folder which can be found in the root dir of this project

## build & start the backend
simply run in a terminal `cargo run` in the projects root dir
