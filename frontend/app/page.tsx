"use client";
import { useState, useRef } from "react";

type CellState = "Empty" | "Hit" | "Miss" | "Ship";
const createEmptyBoard = () => Array.from({ length: 10 }, () => Array(10).fill("Empty"));

export default function GamePage() {
  const [gameId, setGameId] = useState<string>("");
  const [status, setStatus] = useState<string>("Disconnected");
  const [winner, setWinner] = useState<string | null>(null);
  
  const [myBoard, setMyBoard] = useState<CellState[][]>(createEmptyBoard());
  const [enemyBoard, setEnemyBoard] = useState<CellState[][]>(createEmptyBoard());

  const socketRef = useRef<WebSocket | null>(null);
  const HTTP_URL = process.env.NEXT_PUBLIC_BACKEND_HTTP || "http://localhost:3000";
  const WS_URL = process.env.NEXT_PUBLIC_BACKEND_WS || "ws://localhost:3000";

  const startBotGame = async () => {
    if (socketRef.current) {
        socketRef.current.close();
        socketRef.current = null;
    }
    setWinner(null);
    setGameId(""); 
    setMyBoard(createEmptyBoard());
    setEnemyBoard(createEmptyBoard());
    setStatus("Connecting...");

    try {
      const res = await fetch(`${HTTP_URL}/create_game`, { method: "POST" });
      const data = await res.json();
      setGameId(data.game_id);
      connectToWebsocket(data.game_id);
    } catch (err) { 
        console.error(err); 
        setStatus("Error creating game"); 
    }
  };

  const connectToWebsocket = (id: string) => {
    if (socketRef.current) return;

    const ws = new WebSocket(`${WS_URL}/ws/${id}`);
    
    ws.onopen = () => setStatus("Battle Active");
    
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);

      if (data.type === "init") {
        setMyBoard(data.board);
      } 
      else if (data.status === "error") {
        console.warn("Invalid Move:", data.message);
      }
      else if (data.status === "success" && data.turn_update) {
        const { user, bot, winner } = data.turn_update;

        if (user) {
            setEnemyBoard(prev => {
                const newBoard = prev.map(row => [...row]);
                newBoard[user.row][user.col] = user.result;
                return newBoard;
            });
        }

        if (bot) {
            setMyBoard(prev => {
                const newBoard = prev.map(row => [...row]);
                newBoard[bot.row][bot.col] = bot.result;
                return newBoard;
            });
        }

        if (winner) {
            setWinner(winner);
            setStatus(`Winner: ${winner}`);
            ws.close();
            socketRef.current = null;
        }
      }
    };
    socketRef.current = ws;
  };

  const handleShot = (row: number, col: number) => {
    if (!socketRef.current || winner) return; 
    socketRef.current.send(`${row},${col}`);
  };

  return (
    <div className="flex flex-col items-center min-h-screen bg-slate-900 text-white font-mono p-5">
      <h1 className="text-3xl mb-8 font-bold text-blue-400 tracking-widest uppercase border-b border-blue-800 pb-2">
        Battleship Command
      </h1>
      {!gameId && (
        <div className="flex flex-col items-center max-w-2xl w-full animate-fade-in">
            {status === "Connecting..." ? (
                <div className="text-yellow-400 text-xl font-bold animate-pulse mt-20">
                    ESTABLISHING SECURE CONNECTION...
                </div>
            ) : (
                <>
                    <div className="bg-slate-800 p-6 rounded border border-slate-600 mb-8 w-full">
                        <h2 className="text-xl text-yellow-500 mb-4 border-b border-slate-600 pb-2">
                             MISSION BRIEFING
                        </h2>
                        <ul className="space-y-2 text-gray-300 text-sm">
                            <li><strong className="text-white">Objective:</strong> Destroy the enemy fleet.</li>
                            <li><strong className="text-white">The Fleet:</strong> 5 Ships (Size 5, 4, 3, 3, 2).</li>
                            <li><strong className="text-white">Win Condition:</strong> First to land <strong>5 Hits</strong> wins.</li>
                            <li className="text-xs text-gray-500 italic mt-2">* Rapid-fire drill mode active.</li>
                        </ul>
                    </div>

                    <button 
                        onClick={startBotGame} 
                        className="px-8 py-3 bg-blue-700 hover:bg-blue-600 rounded font-bold text-lg transition"
                    >
                        DEPLOY FLEET
                    </button>
                </>
            )}
        </div>
      )}
      {gameId && (
        <div className="flex flex-col items-center gap-8 w-full max-w-5xl">
            <div className="flex items-center justify-between w-full bg-slate-800 p-3 rounded border border-slate-700">
                <div className={`px-4 py-1 rounded font-bold text-sm uppercase tracking-wider
                    ${winner === "User" ? "bg-green-700 text-white" : ""}
                    ${winner === "Bot" ? "bg-red-700 text-white" : ""}
                    ${!winner ? "text-blue-200" : ""}
                `}>
                    STATUS: {winner ? `${winner} VICTORY` : status}
                </div>
                
                <button 
                    onClick={startBotGame}
                    className="px-4 py-1 bg-slate-700 hover:bg-slate-600 border border-slate-500 rounded text-sm text-gray-200"
                >
                    {winner ? "NEW MISSION" : "RESTART"}
                </button>
            </div>

            <div className="flex flex-col md:flex-row gap-12 justify-center">
                <div>
                    <h2 className="text-lg mb-2 text-center text-blue-300">Your Sector</h2>
                    <div className="grid grid-cols-10 gap-1 bg-slate-800 p-2 rounded border border-blue-900">
                    {myBoard.map((row, r) => row.map((cell, c) => (
                        <div
                        key={`my-${r}-${c}`}
                        className={`
                            w-8 h-8 border border-slate-600 flex items-center justify-center 
                            ${cell === "Empty" ? "" : ""}
                            ${cell === "Ship" ? "bg-blue-500" : ""}
                            ${cell === "Hit" ? "bg-red-500" : ""}
                            ${cell === "Miss" ? "bg-slate-500" : ""}
                        `}
                        />
                    )))}
                    </div>
                </div>
                <div className={winner ? "opacity-50 pointer-events-none grayscale" : ""}>
                    <h2 className="text-lg mb-2 text-center text-red-300">Enemy Sector</h2>
                    <div className="grid grid-cols-10 gap-1 bg-slate-800 p-2 rounded border border-red-900">
                    {enemyBoard.map((row, r) => row.map((cell, c) => (
                        <div
                        key={`enemy-${r}-${c}`}
                        onClick={() => handleShot(r, c)}
                        className={`
                            w-8 h-8 border border-slate-600 flex items-center justify-center 
                            ${!winner ? "cursor-pointer hover:bg-slate-700" : ""}
                            ${cell === "Hit" ? "bg-red-500" : ""}
                            ${cell === "Miss" ? "bg-slate-500" : ""}
                        `}
                        />
                    )))}
                    </div>
                    {!winner && (
                        <div className="text-xs text-center text-gray-500 mt-2">
                            CLICK TO FIRE
                        </div>
                    )}
                </div>
            </div>
            <div className="flex gap-6 text-xs text-gray-400 mt-2 border-t border-slate-700 pt-4">
                <div className="flex items-center"><span className="w-3 h-3 bg-blue-500 mr-2 border border-slate-600"></span>Ship</div>
                <div className="flex items-center"><span className="w-3 h-3 bg-red-500 mr-2 border border-slate-600"></span>Hit</div>
                <div className="flex items-center"><span className="w-3 h-3 bg-slate-500 mr-2 border border-slate-600"></span>Miss</div>
            </div>

        </div>
      )}
    </div>
  );
}