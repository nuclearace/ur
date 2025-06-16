# Royal Game of Ur

A fast, feature-rich implementation of the ancient Royal Game of Ur written in Rust, featuring multiple AI opponents and comprehensive statistics.

## About the Game

The Royal Game of Ur is one of the oldest known board games, dating back to ancient Mesopotamia (c. 2600 BCE). This race game is considered an ancestor of modern backgammon and combines elements of luck and strategy.

### Game Rules

- **Players**: 2 players, each controlling 7 pieces
- **Objective**: Be the first to move all 7 pieces through the board and off the exit
- **Dice**: Roll 4 binary dice (0-4 moves per turn)
- **Board**: 20 squares total with a unique path for each player
- **Capture**: Land on opponent's piece in shared squares to send it back to start
- **Safe Squares**: 5 squares where pieces cannot be captured
- **Rosettes**: 3 special squares that grant an extra turn and protection from capture
- **Winning**: Pieces must roll exactly the right number to exit the board

## Features

- 🎯 **Multiple Game Modes**: Human vs Human, Human vs AI, AI vs AI
- 🤖 **Three AI Types**:
  - Random AI (chaotic fun)
  - Smart AI (strategic heuristics)
  - MCTS AI (Monte Carlo Tree Search with multithreading)
- 📊 **Statistics Mode**: Run bulk simulations to analyze AI performance
- 🎨 **Beautiful Terminal UI**: Colorful board display with Unicode symbols
- ⚡ **Optimized Performance**: Fast game state representation for rapid simulations
- 🧵 **Multithreaded MCTS**: Configurable threading for AI performance

## Installation & Setup

### Prerequisites
- Rust (latest stable version)
- Terminal with Unicode support

### Building
```bash
git clone <repository-url>
cd ur
cargo build --release
```

### Running
```bash
cargo run --release
```

## Game Modes

When you start the game, you'll be presented with these options:

- **[0]** Watch two smart AI bots play against each other
- **[1]** Play against smart AI (you are Player 1)
- **[2]** Two human players
- **[3]** Watch random AI vs smart AI
- **[4]** Statistics - Run multiple games and show results
- **[5]** Play against MCTS AI (you are Player 1)
- **[6]** Watch MCTS AI vs Smart AI
- **[7]** Watch two MCTS AI bots play against each other

## AI Performance

The project includes three different AI implementations:

### Random AI 🎲
- Makes completely random legal moves
- Good for testing and casual play
- Fastest execution

### Smart AI 🧠
- Uses strategic heuristics
- Considers piece safety, advancement, and capture opportunities
- Balances offense and defense
- Fast and competitive

### MCTS AI 🤖
- Monte Carlo Tree Search with configurable simulations
- Multithreaded for maximum performance
- Strongest player but requires more computation time
- Configurable simulation count and thread pool

## Statistics Mode

Run comprehensive AI matchups to analyze performance:
- Configure number of games to simulate
- Compare different AI strategies
- View win rates and performance metrics
- Useful for AI development and analysis

## Board Layout

```
Player 1 Path: [1][2][3][4]
                        [5]
Shared Path:           [6][7][8][9][10][11][12][13]
                        [14]
Player 2 Path: [15][16][17][18]
```

- **Squares 1-4, 15-18**: Player-specific safe zones
- **Squares 6-13**: Shared combat zone where captures can occur
- **Squares 5, 14**: Safe transition squares
- **Rosettes**: Special squares at positions 4, 8, 14 (grant extra turns)

## Performance Features

- **Bitboard Representation**: Compact game state for fast copying and comparison
- **Move Generation**: Efficient legal move calculation
- **Parallel MCTS**: Configurable multithreading for AI calculations
- **Optimized Simulation**: Thousands of games per second for statistics

## Controls

### Human Players
- Press **ENTER** to roll dice
- Select moves by entering the corresponding number
- Follow on-screen prompts for piece selection

### AI Configuration
- Choose whether to use multithreading for MCTS
- Configure number of threads (defaults to CPU core count)
- Adjust simulation counts for AI strength vs speed tradeoff

## Dependencies

- `crossterm`: Cross-platform terminal manipulation
- `std`: Standard Rust library (threading, I/O, etc.)

## Development

The codebase is organized into several modules:

- `main.rs`: Game loop and user interface
- `optimized_game.rs`: Fast game state representation
- `ai.rs`: MCTS AI implementation
- `ai_helpers.rs`: Random and Smart AI implementations
- `display.rs`: Terminal UI and board rendering
- `stats.rs`: Statistics and bulk simulation mode

---
