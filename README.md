# Unlimited Poker
![Crab](./assets/crab.png)

### WIP, a poker game server backend written in rust using the following dependencies:
- Actix web service framework
- async-graphql GraphQL server library
- push based architecture using web socket
- Coming soonest: k8s containerized deployment using in memory
- Coming soon: NoSql database (MongoDB or Redis)
- Coming soon: RL trained players
- Coming soon: kafka and/or flink for distributed streaming and horizontal scaling of stateless real-time eventing
- Coming soon: react front end



### Usage
- see instructions to play using graphiql graphql playground: [play](./play.md)
### Deal Client payload
```rust
cargo run deal --player-count 3
   Compiling deal_client v0.1.0 (/Users/seanglover/Development/deal_client)
    Finished dev [unoptimized + debuginfo] target(s) in 0.83s
     Running `target/debug/deal_client deal --player-count 3`
Requesting deal for player count: 3
deal request: Request {
    metadata: MetadataMap {
        headers: {},
    },
    message: HandRequest {
        player_count: 3,
    },
    extensions: Extensions,
}
Requested deal: HandResponse {
    board: Some(
        Board {
            flop: [
                "6c",
                "Ac",
                "8d",
            ],
            turn: "7d",
            river: "5s",
        },
    ),
    hands: [
        Hand {
            cards: [
                "2h",
                "Ad",
            ],
            score: 0.5294827124095417,
            description: "Pair",
        },
        Hand {
            cards: [
                "5d",
                "5c",
            ],
            score: 0.7039667649423746,
            description: "Three of a Kind",
        },
        Hand {
            cards: [
                "Qh",
                "4h",
            ],
            score: 0.7847761994103457,
            description: "Straight",
        },
    ],
}
```

## Setup local
```
docker-compose up
npm install -g dynamodb-admin
dynamodb-admin
```

### Database Init
```bash
aws dynamodb create-table --table-name tables --attribute-definitions AttributeName=id,AttributeType=S --key-schema AttributeName=id,KeyType=HASH --provisioned-throughput ReadCapacityUnits=5,WriteCapacityUnits=5 --endpoint-url http://localhost:8000 --region us-east-2

aws dynamodb create-table --table-name hands --attribute-definitions AttributeName=id,AttributeType=S --key-schema AttributeName=id,KeyType=HASH --provisioned-throughput ReadCapacityUnits=5,WriteCapacityUnits=5 --endpoint-url http://localhost:8000 --region us-east-2

aws dynamodb create-table --table-name users --attribute-definitions AttributeName=id,AttributeType=S --key-schema AttributeName=id,KeyType=HASH --provisioned-throughput ReadCapacityUnits=5,WriteCapacityUnits=5 --endpoint-url http://localhost:8000 --region us-east-2
```

```
docker exec -it c313d60605ed mongo -u root -p example poker

db.hands.findOne({ "id": "58981fe2-1476-4fe9-b9ec-091030b829c2" })

```