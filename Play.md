# Play

1. All players subscribe to deal event with user and table token:
2. Deal Event is called manually (TODO: triggered by some as yet undetermined threshhold)
3. Players subscribe to hand event channel with user and hand token
4. Players send events for the hand in order based on actions received from the channels
5. Players receive player events, street change events, and hand/table over events


### 1. deal event
```gql
subscription DealSubscription($mutationType: MutationType) {
  deal(mutationType: $mutationType) {
    mutationType
    id
    deal {
      id
      tableId
      players {
        id
        stack
        cards
        score
        description
      }
      cards {
        flop
        turn
        river
      }
      playerEvents {
        playerId
        action
        amount
        streetType
        currentStack
        currentPot
      }
      streetEvents {
        streetType
        currentActivePlayers {
          id
          bet
          stack
          isInactive
        }
        pot
      }
    }
  }
}
```

```json
{
  "x-user-token": "sean",
  "x-table-token": "123"
}
```

### 2. deal call
```gql
mutation DealHand($dealInput: DealInput!) {
    deal(dealInput: $dealInput)
}
```

```json
{
  "dealInput": {
    "players": [
      {
        "id": "sean",
        "stack": 1000.0
      },
      {
        "id": "neuro",
        "stack": 1000.0
      },
      {
        "id": "pali",
        "stack": 1000.0
      }
    ],
    "tableId": "table123"
  }
}
```

```json
{
  "x-user-token": "sean",
  "x-table-token": "123"
}
```

### 3. hand event
```graphql
subscription OnHandEvent($mutationType: MutationType) {
  handEvent(mutationType: $mutationType) {
    mutationType
    handId
    streetEvent {
      streetType
      currentActivePlayers {
        id
        bet
        stack
        isInactive
        isBigBlind
      }
      pot
    }
    playerEvent {
      playerId
      action
      amount
      streetType
      currentStack
      currentPot
    }
    cards {
      flop
      turn
      river
    }
  }
}
```

```json
{
  "x-user-token": "sean",
  "x-table-token": "123",
  "x-hand-token": "0"
}
```


### 3. play (hand event)
```gql
mutation PlayTurn($id: ID!, $playerId: ID!, $action: PlayerAction!, $amount: Decimal!) {
  playTurn(id: $id, playerId: $playerId, action: $action, amount: $amount) {
    id
    stack
    isInactive
    isBigBlind
    pot
    playerEvent {
      playerId
      action
      amount
      streetType
      currentStack
      currentPot
    }
    cards {
      flop
      turn
      river
    }
  }
}
```

```json
{
  "id": "0",
  "playerId": "sean",
  "action": "BET",
  "amount": 10.0
}
```

```json
{
  "x-user-token": "sean",
  "x-hand-token": "0"
}
```

