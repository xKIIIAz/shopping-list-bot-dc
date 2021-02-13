use serenity::{
    async_trait,
    model::{channel::Message,
            gateway::Ready,
            channel::ReactionType,
            id::GuildId},
    prelude::*,
    utils::MessageBuilder,
};

use std::io::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

/*
 * example command to build for android device
 * cargo ndk --platform 21 --target armv7-linux-androideabi build
 */ 

 /*
  * ProductsLists is a key for a typemap which is assigned to a Client.data field.
  * ProductsLists contains a HashMap <K = GuildID, V = Vec<String>>
  * where K is an ID of a Guild for which the V is stored 
  *       V is a vector of products added to the shopping list
  *
  */
struct ProductsLists;
impl TypeMapKey for ProductsLists { 
    type Value = HashMap<GuildId, Vec<String>>; 
}
struct Handler;
impl Handler {

    /*
     * Function determines which command is called and returns string which distinguishes commands
     */
    async fn determine_command(&self, ctx: Context, msg: Message) -> &str{
        let message = msg.content.clone();

        let gid = msg.guild_id.unwrap();// it should be handled better because, a message can come from a user (in private chat);/

        if message.starts_with("!"){
            let mut products: Vec<String>;
            let mut command = message.split_whitespace();
            let word = command.next().unwrap();
            if word.contains("shop") {
                let subcommand = command.next().unwrap();
                products = self.get_products(ctx.clone(), gid).await;
                if subcommand.contains("new") {
                    //self.update_history(&products); //the error exception should be handled
                    products.clear();
                    self.save_products(ctx, gid, &products).await; 
                    return "created";
                }
            } else if word.contains("bought") {
                let subcommand = command.next().unwrap();
                match subcommand.parse::<i32>() {
                    Ok(num) => {
                        let mut products = self.get_products(ctx.clone(), gid).await;
                        let n = num-1;
                        if n >= 0 && products.len() as i32 > n {
                            products.remove(n as usize);
                        }
                        self.save_products(ctx,gid,&products).await;
                        return "bought"
                    },
                    Err(_) => return "unknown",
                };
            } else if word.contains("help") {
               return "help"; 
            }
        }
        if message.starts_with("-") {
            let mut products: Vec<String> = self.get_products(ctx.clone(), gid).await;
            let mut prod = message.chars();
            let mut product: String = String::new();
            prod.next();
            for x in prod {
                product.push_str(&x.to_string());
            }
            products.push(product);
            self.save_products(ctx, gid, &products).await;
            return "added";
        }
        return "unknown";
    }


    async fn get_products(&self, ctx: Context, guild_id: GuildId) -> std::vec::Vec<String> {
        match ctx.data.read().await.get::<ProductsLists>().unwrap().get(&guild_id) {
            Some(some) => return some.to_vec(),
            None => return Vec::new(),
        };
    }

    async fn save_products(&self, ctx: Context, guild_id: GuildId, products: &[String])  {
        let mut data = ctx.data.write().await;
        let guilds_lists = data.get_mut::<ProductsLists>().unwrap();
        
        guilds_lists.insert(
            guild_id,
            products.to_vec()
        );

    }
    //Here the bot builds the message with the list of functions. Change it to your liking.
    async fn send_help(&self, ctx: Context, msg: Message) {
        if let Err(why) = msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Nadeszła pomoc!");
                e.description("\"!shop new\" - tworzy nową pustą listę\n\"-nazwa produktu\" - dodaje produkt do listy\n\"!bought <numer>\" - usuwa pozycję <numer> z listy.");
                e
            });
            m
        }).await {
            println!("Could not send a list message: {}", why);
        }
    }
    async fn send_shopping_list(&self, ctx: Context, msg: Message) {
        let data = ctx.data.read().await;
        let saved_products = data.get::<ProductsLists>().unwrap().get(&msg.guild_id.unwrap()).unwrap();
        let mut list: String = String::new();
        for i in 0..saved_products.len() {
            list.push_str((i+1).to_string().as_str());
            list.push_str(") ");
            list.push_str(saved_products[i].as_str());
            list.push_str("\n");
        }
        //Here the bot builds the message with shopping list. Change title and footer to your liking.
        if let Err(why) = msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Nasza Lista Zakupów!");
                e.description(list.as_str());
                e.footer(|f| {
                    f.text("Aby dodać rzeczy do listy, wpisz \"-nazwa produktu\"");
                    f
                });
                e
            });
            m
        }).await {
            println!("Could not send a list message: {}", why);
        }
    }
    /*
     * Below are three functions responsible for adding a reaction emoji to message with command. And starting 
     */
    async fn react_created(&self, ctx: Context, msg: Message) {
        if let Err(why) = msg.react(&ctx.http, ReactionType::Unicode("👌".to_string())).await {
            println!("Could not add reaction to the message! {:?}", why);
        }
        let response = MessageBuilder::new()
                .push_bold_safe(&msg.author.name)
                .push(" created a new shopping list! ")
                .build();

        if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
            println!("Error sending message: {:?}", why);
        } 
        self.send_shopping_list(ctx, msg).await;
    }
    async fn react_added(&self, ctx: Context, msg: Message) {
        if let Err(why) = msg.react(&ctx.http, ReactionType::Unicode("💸".to_string())).await {
            println!("Could not add reaction to the message! {:?}", why);
        }
        self.send_shopping_list(ctx, msg).await;
    }
    async fn react_bought(&self, ctx: Context, msg: Message) {
        if let Err(why) = msg.react(&ctx.http, ReactionType::Unicode("✔".to_string())).await {
            println!("Could not add reaction to the message! {:?}", why);
        }
        self.send_shopping_list(ctx,msg).await;
    }

}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if msg.content.starts_with("!") || msg.content.starts_with("-") {
            let command = self.determine_command(context.clone(), msg.clone()).await;
            match command {
                "help" => self.send_help(context, msg).await,
                "created" => self.react_created(context, msg).await,
                "added" => self.react_added(context, msg).await,
                "bought" => self.react_bought(context, msg).await,
                "unknown" => println!("Received unknown command!"),
                _ => println!("Impossible o_o"),
            };
        }
    } 

    
    async fn ready(&self, _: Context, ready: Ready) {
        //here are all the things the bot does when it starts up
        println!("{} is connected!", ready.user.name);
    }
}

fn get_token() -> String {
    let mut contents = String::new();
    match std::fs::File::open("token.json") {
        Ok(f) => {
            let mut br = std::io::BufReader::new(f);
            match br.read_to_string(&mut contents) {
                Ok(o) => o,
                Err(why) => panic!("Could not read the token file!: {}", why),
            };
        },
        Err(_) => println!("Could not read the file"),
    };
    let token: Value = serde_json::from_str(&contents).expect("JSON failed.");
    let token: String = match token["token"].as_str(){
        Some(string) => (*string).to_string(),
        None => panic!("Empty token!")
    };
    return token;
}
#[tokio::main]
async fn tokiomain() {
    let mut client = Client::new(get_token())
        .event_handler(Handler).await.expect("Err: Creating a client.");
    {
    //preparing the data field
       let mut data = client.data.write().await;
       data.insert::<ProductsLists>(HashMap::new());
    }//here the lock is dropped
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
fn main() {
    //creates an asynchronous instance to work with discord API
    /*let tokio = */std::thread::spawn(move || {
        tokiomain();
    });
    //main thread waits for signal to end the execution
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => {
            std::process::exit(0);
        },
        Err(e) => panic!("Error reading a string from stdin: {}", e),
    };
    
}

/* TODO:
 */