# Solana Layer2 node

A Solana based Layer 2 node. 

## Building

For how to build and test code, see [solana]([solana/README.md at master · solana-labs/solana · GitHub](https://github.com/solana-labs/solana/blob/master/README.md))'s succinct instructions.

### Design

The architecture of design as below:

![Architecture](./architecture-diagram.svg)

There are two roles in our Layer 2: __Execution Layer__ and __Settlement Layer__.

* Execution Layer contains:
  
  * Execution Node
    
    Handles all Layer 2 transactions, produce block, push block to DA. 
  
  * Verification Node:
    
    Pull data from DA, reconstruct block and replay.

* Settlement Layer contains:
  
  * Full node:
    
    Check the challenge and push transaction data to DA.
  
  * Light node:
    
    Sync header data.

## Progress

1. Execution Layer
   
   ```mermaid
   graph TD
       A[Execution Node] --> |push block| B[DA]
       B --> |replay data| C[Verification Node]
   
   ```
   
   For now, since Celestia is still unstable, we use PostgreSQL as a DA simulator, here is the execution flow:
   
   * The execution node produce blocks, we use [accountsdb-plugin-postgres](https://github.com/EulerSmile/solana-accountsdb-plugin-postgres) to save blocks into PostgreSQL database.
   
   * The Verification node query blocks from database, reconstruct them and do replay.

2. Settlement Layer

    TBD
