# Chess Application

A full-stack chess application built with a serverless backend and a modern frontend. It allows two players to play a game of chess in real-time.

## Deployed Application

You can play at: [https://chess.brendandagys.com](https://chess.brendandagys.com)

## Features

- **Simultaneous Games**: Players can play in multiple games at the same time.
- **Customizable Boards**: The game supports custom board sizes and multiple piece set-ups for a unique chess experience.

## Chess Rules Implemented

The application enforces the standard rules of chess, including:

- **Pawn Promotion**: Pawns that reach the opposite end of the board are promoted.
- **En Passant**: The special "en passant" pawn capture is implemented.
- **Castling**: Players can castle to protect their king and connect their rooks.
- **Check, Checkmate, and Stalemate**: The game accurately detects and manages check, checkmate scenarios to conclude the game.

## Principles Used

- **Serverless Architecture**: The back-end is built using AWS SAM, Lambda, and DynamoDB, which allows for a scalable and extremely cost-effective deployment.
- **Single Page Application (SPA)**: The front-end is a React-TypeScript application built with Vite.
- **Real-time Communication**: WebSockets are used for real-time communication between the players and the server.

## Technology Stack

### Backend

- **AWS SAM**: Infrastructure as Code for serverless applications.
- **AWS Lambda**: Serverless compute functions written in Rust.
- **Amazon DynamoDB**: NoSQL database for storing game state.
- **Amazon API Gateway**: Manages WebSocket connections and API endpoints.
- **Rust**: The programming language used for the Lambda functions.

### Frontend

- **React**: A JavaScript library for building user interfaces.
- **TypeScript**: A typed superset of JavaScript.
- **Vite**: A fast build tool for modern web development.
- **WebSockets**: For real-time communication with the backend.
