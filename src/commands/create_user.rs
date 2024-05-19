use futures_util::TryStreamExt;
use tiberius::QueryItem;
use tiberius::{Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use uuid::Uuid;

static ALL_FEATURES_ROLE_ID: &str = "FE4DA52F-2104-424D-B1C1-B07400E64A44";

pub(crate) async fn invoke(name: &str, email: &str, connection_string: &str) {
    println!(
        "Trying to connect to database with connection string: {}",
        connection_string
    );
    let mut client = init_client(connection_string)
        .await
        .expect("Could not connect to database");

    if user_with_email_exists(&mut client, email)
        .await
        .expect("Could not check if user exists")
    {
        eprintln!("User with email {} already exists", email);
        return;
    }

    println!("Adding user {} with email {}", name, email);
    let user_id = insert_user(&mut client, name, email)
        .await
        .expect("Could not insert user");
    println!("Connecting user to role");
    add_role_to_user(&mut client, &user_id, ALL_FEATURES_ROLE_ID)
        .await
        .expect("Could not add role to user");

    println!("Created user {} with email {}", name, email);
}

async fn insert_user(
    client: &mut Client<Compat<TcpStream>>,
    name: &str,
    email: &str,
) -> Result<String, tiberius::error::Error> {
    let (first_name, last_name) = name.split_once(' ').unwrap_or(("", ""));
    let unique_id = Uuid::new_v4();

    let query = r#"INSERT INTO [dbo].[User]
            ([Id]
            ,[FirstName]
            ,[LastName]
            ,[Email]
            ,[Username]
            ,[Timezone]
            ,[Locale]
            ,[Active]
            ,[OfficeId]
            ,[SystemUser]
            ,[DepartmentId]
            ,[LastLogin]
            ,[ActivationChanged])
        VALUES
            (@P1
            ,@P2
            ,@P3
            ,@P4
            ,@P4
            ,'W. Europe Standard Time'
            ,'sv-SE'
            ,1
            ,NULL
            ,0
            ,NULL
            ,NULL
            ,NULL)"#;

    match client
        .execute(query, &[&unique_id, &first_name, &last_name, &email])
        .await
    {
        Ok(_) => {
            println!("User created successfully");
            Ok(unique_id.to_string())
        }
        Err(e) => {
            eprintln!("Could not create user: {}", e);
            Err(e)
        }
    }
}

async fn add_role_to_user(
    client: &mut Client<Compat<TcpStream>>,
    user_id: &str,
    role_id: &str,
) -> Result<(), tiberius::error::Error> {
    let query = r#"INSERT INTO [dbo].[UsersToRoles]
            ([UserId]
            ,[UserRoleId])
        VALUES
            (@P1
            ,@P2)"#;

    match client.execute(query, &[&user_id, &role_id]).await {
        Ok(_) => {
            println!("Role added to user successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("Could not add role to user: {}", e);
            Err(e)
        }
    }
}

async fn user_with_email_exists(
    client: &mut Client<Compat<TcpStream>>,
    email: &str,
) -> Result<bool, tiberius::error::Error> {
    let query = r#"SELECT COUNT(*)
        FROM [dbo].[User]
        WHERE [Email] = @P1"#;

    let mut stream = client.query(query, &[&email]).await?;
    let mut exists = false;
    while let Some(item) = stream.try_next().await? {
        match item {
            QueryItem::Row(row) if row.result_index() == 0 => {
                row.get::<i32, _>(0)
                    .map(|count| exists = count > 0)
                    .expect("Could not get count");
            }
            _ => {}
        }
    }

    Ok(exists)
}

async fn init_client(
    connection_string: &str,
) -> Result<Client<Compat<TcpStream>>, tiberius::error::Error> {
    let config = Config::from_ado_string(connection_string)?;

    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let client = Client::connect(config, tcp.compat_write()).await?;

    Ok(client)
}
