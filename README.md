## Installation
```powershell
powershell -c "irm raw.githubusercontent.com/ponbac/tainer-cli/master/install.ps1 | iex"
```

This will install the tool under your home directory and append it to your path, making the `tainer` command available globally.

## Commands
- `connection-strings`  Sets your connection strings everywhere it needs to be set
- `git`                 Run a command against each git repository
- `application-host`    Allow authentication in applicationhost.config
- `web-api`             Fix Azure auth in Web API appsettings
- `create-user`         Create a new user in database, with an attached role
- `help`                Print this message or the help of the given subcommand(s)

## Examples
### connection-strings
Will append your connection strings to all `app.config` and `web.config` files. This command also searches for `appsettings.json` and creates a development copy (`appsettings.Development.json`) with your connection strings.
```powershell
# tainer connection-strings <COMPUTER_NAME> <MAIN_DB_CONNECTION_STRING> <SERVICE_BUS_CONNECTION_STRING>
tainer connection-strings PINKGOLD "Data Source=PINKGOLD\PINKGOLD16;Initial Catalog=dbEnvirotainerELOS;Integrated Security=SSPI;" "Data Source=PINKGOLD\PINKGOLD16;Initial Catalog=EnvirotainerNServiceBus;Integrated Security=SSPI;"
```

### git
Executes any `git` command against all repositories recursively below your current directory. This is done in parallel and errors are ignored, in comparison to `git submodule foreach --recursive` where execution is sequential and one failure stops the entire process.
```powershell
# checkout project/VQT in all repos where the branch exists
tainer git checkout project/VQT
# pull the latest changes in every branch
tainer git pull
```

### create-user
Creates a new user with the given name and email in the provided database. The user will also be given the role `ALL FEATURES`.
```powershell
# tainer create-user <FULL_NAME> <EMAIL> <DB_CONNECTION_STRING>
tainer create-user "Pontus Backman" pontus.backman@spinit.se "Data Source=PINKGOLD\PINKGOLD16;Initial Catalog=dbEnvirotainerELOS;Integrated Security=SSPI;"
```

### application-host
Enables authentication in your `applicationhost.config`.
```powershell
tainer application-host
```

### web-api
Adds the correct Azure AD settings to your appsettings for the WebAPI-project.
```powershell
tainer web-api
```