let cheatMode = false;

const overContainer = document.getElementById('game-over-container');
const playerId = document.getElementById("player");
const opponentId = document.getElementById("opponent");
const scoreboardDescription = document.getElementById('scoreboard-description')
const playerColorShort = document.getElementById("hidden-color").value;

const game = new Chess();

const playerColor = playerColorShort === "w" ? "white" : "black";

const boardConfig = {
    draggable: true,
    dropOffBoard: 'snapback',
    moveSpeed: 'slow',
    snapbackSpeed: 'slow',
    snapSpeed: 'slow',
    orientation: playerColor,
    position: game.fen(),
    onDragStart,
    onDrop,
};
const board = Chessboard2('board', boardConfig);

function isPiecePlayerColor(piece) {
    return new RegExp(`^${playerColorShort}`).test(piece);
}

function onDragStart(dragStartEvt) {
    if (game.turn() !== playerColorShort || !isPiecePlayerColor(dragStartEvt.piece)) return false;
    game.moves({square: dragStartEvt.square, verbose: true}).forEach(move => board.addCircle(move.to));
}

function onDrop(dropEvt) {
    const move = game.move({
        from: dropEvt.source, to: dropEvt.target, promotion: 'q',
    });
    board.clearCircles();
    if (!move) return 'snapback';
    board.position(game.fen());
    opponentMove(dropEvt.source, dropEvt.target);
    checkGameOver();
}

async function fetchBoard(src, dest) {
    const response = await fetch("/move", {
        method: "POST", cache: "no-cache", headers: {
            "Content-Type": "text/plain"
        }, body: src + dest
    });
    return await response.text();
}

async function opponentMove(src, dest) {
    if (game.turn() === playerColorShort) return;
    try {
        const fen = await fetchBoard(src, dest);
        game.load(fen);
        board.position(game.fen());
    } catch (e) {
        console.error(e);
    }
    await checkGameOver();
    if (cheatMode) await cheat();
}

async function checkGameOver() {
    if (!game.game_over()) {
        highlightTurn();
        return;
    }

    const response = await fetch('/game_end');
    if (!response.ok) {
        if (response.status === 406) {
            const fen = await response.text();
            game.load(fen);
            board.position(game.fen());
        } else {
            console.error('ERROR: Unexpected Status while fetching game over: ' + response.status);
        }
        highlightTurn()
        return;
    }

    let overText = '';
    if (game.in_checkmate()) overText = (game.turn() === 'w' ? 'White' : 'Black') + ' is Checkmate';
    if (game.in_draw()) overText = 'Draw';
    document.getElementById('over-description').innerText = overText;

    fetchScoreboard(1000).then(data => renderScoreboard(data));
    overContainer.style.display = "block";
}

function highlightTurn() {
    playerId.style.opacity = game.turn() === playerColorShort ? '100%' : '50%';
    opponentId.style.opacity = game.turn() !== playerColorShort ? '100%' : '50%';

    if (game.in_check()) {
        const kingCords = [].concat(...game.board()).map((p, index) => {
            if (p && p.type === game.KING && p.color === game.turn()) return index;
        }).filter(Number.isInteger).map(piece_index => {
            const row = 'abcdefgh'[piece_index % 8];
            const column = Math.ceil((64 - piece_index) / 8);
            return row + column;
        });

        const chessSquare = document.querySelector(`div[data-square-coord="${kingCords}"]`);
        if (chessSquare) chessSquare.classList.add("highlight-in-chess");
    } else {
        const highlight = document.querySelector(".highlight-in-chess");
        if (highlight) highlight.classList.remove("highlight-in-chess");
    }
}

async function cheat() {
    if (game.game_over()) return;

    const response = await (await fetch("https://chess-api.com/v1", {
        method: "POST", headers: {
            "Content-Type": "application/json"
        }, body: JSON.stringify({fen: game.fen()}),
    })).json();

    game.move(response['san']);
    board.position(game.fen());

    await opponentMove(response['move'], "");
    await checkGameOver();
}

async function fetchScoreboard(count) {
    try {
        const response = await fetch(`/scoreboard?count=${count}`);
        if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
        return await response.json();
    } catch (error) {
        console.error('Failed to fetch scoreboard:', error);
    }
}

function renderScoreboard(scoreboard) {
    const scoreboardDiv = document.getElementById('scoreboard');
    scoreboardDiv.innerHTML = '';

    if (scoreboard && scoreboard.length > 0) {
        const table = document.createElement('table');
        table.classList.add('scoreboard-table');
        const thead = document.createElement('thead');
        const tbody = document.createElement('tbody');

        const headerRow = document.createElement('tr');
        ['Rank', 'Winner', 'Score'].forEach(text => {
            const th = document.createElement('th');
            th.textContent = text;
            headerRow.appendChild(th);
        });
        thead.appendChild(headerRow);

        scoreboard.forEach((entry, index) => {
            const row = document.createElement('tr');
            [index + 1, entry.winner, entry.score].forEach(text => {
                const td = document.createElement('td');
                td.textContent = text;
                row.appendChild(td);
            });
            tbody.appendChild(row);
        });

        table.appendChild(thead);
        table.appendChild(tbody);
        scoreboardDiv.appendChild(table);

        scoreboardDescription.textContent = 'Take a look at the Scoreboard:';
    } else {
        scoreboardDescription.textContent = 'There are no Scores yet.';
    }
}

function firstMove() {
    highlightTurn();
    if (playerColorShort === "b") opponentMove("0", "0"); else if (cheatMode) cheat();
}

firstMove();
