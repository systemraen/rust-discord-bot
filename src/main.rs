use {
    serenity::{
        async_trait, builder,
        client::{Client, Context, EventHandler},
        framework::standard::{
            help_commands,
            macros::{command, group, help, hook},
            Args, CommandError, CommandGroup, CommandResult, Delimiter, HelpOptions,
            StandardFramework,
        },
        http::Http,
        model::{channel::Message, id::UserId},
        prelude::*,
    },
    std::{
        collections::{HashMap, HashSet},
        env,
    },
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[help]
#[individual_command_tip = "Hello! こんにちは！Hola! Bonjour! 您好!\n\
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn my_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    //msg.reply(ctx, format!("{:?}", groups)).await?;
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn unknown_command(ctx: &Context, msg: &Message, _cmd: &str) {
    if msg.content.starts_with('!') {
        if let Err(why) = msg
            .reply(ctx, "bzzz... don't know that one :pensive:")
            .await
        {
            println!("Error occured on unknown_command reply: {:?}", why);
        };
    }
}

struct JoinableRoles;
impl TypeMapKey for JoinableRoles {
    type Value = Vec<String>;
}

//https://stackoverflow.com/a/38183903/349575
macro_rules! vec_of_strings {
    ($($x:expr), *) => (vec![$($x.to_string()), *]);
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCO_MONK_TOKEN").expect("Missing token from environment");
    let http = Http::new_with_token(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefix("!")
                .delimiters(vec![", ", ","])
                .owners(owners)
        })
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .unrecognised_command(unknown_command);

    // Login with a bot token from the environment
    let mut client = Client::new(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<JoinableRoles>(vec_of_strings![
            "bot watchers",
            "politics",
            "makers",
            "venters"
        ]);
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[group]
#[commands(ping, pong, list, join, drop, play, loop_sound)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn pong(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Ping!").await?;

    Ok(())
}

#[command]
#[sub_commands(roles)]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    // todo: list subcommands

    Ok(())
}

#[command]
async fn roles(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let roles = data
        .get::<JoinableRoles>()
        .expect("Unable to get JoinableRoles from context");

    msg.reply(ctx, format!("{:#?}", roles)).await?;

    Ok(())
}

#[command]
async fn join(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let joinable_roles = data
        .get::<JoinableRoles>()
        .expect("Unable to get JoinableRoles from context");

    if let Some(member) = msg.member(&ctx.cache).await {
        if let Some(guild) = msg.guild(&ctx.cache).await {
            let mut to_join = Vec::new();
            let mut join_names = Vec::new();

            to_join.extend_from_slice(&member.roles);

            for arg in args.iter::<String>() {
                let potential_role_name = arg.expect("");
                if let Some(role) = guild.role_by_name(&potential_role_name) {
                    if joinable_roles.contains(&potential_role_name) {
                        to_join.push(role.id);
                        join_names.push(potential_role_name.clone());
                    }
                }
            }

            #[cfg(debug_assertions)]
            println!("Joining: \n{:?}\n{:?}", &join_names, &to_join);

            if let Err(why) = guild
                .edit_member(&ctx.http, msg.author.id, |m| m.roles(&to_join))
                .await
            {
                #[cfg(debug_assertions)]
                println!("Edit Member Failed: {}", why);
            } else {
                msg.reply(&ctx.http, format!("Joining {:?}", join_names))
                    .await?;
            }
        }
    }

    Ok(())
}



#[command]
async fn drop(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let roles = data
        .get::<JoinableRoles>()
        .expect("Unable to get JoinableRoles from context");
    let potential_role_name = args.rest();
    let arg_split = potential_role_name.split(",");

    for role in arg_split {
        if roles.contains(&role.to_string()) {
            // todo: check if user is actually in role
            msg.reply(ctx, format!("Leaving {}", role)).await?;
        }
    }

    Ok(())
}

#[command]
async fn play(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
async fn loop_sound(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}
