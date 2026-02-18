"use client";

import React, { useState, useRef, useEffect, useCallback } from "react";

type CellState = "Empty" | "Hit" | "Miss" | "Ship";
type Difficulty = "Easy" | "Medium" | "Hard";
type GameStatus =
  | "Disconnected"
  | "Connecting..."
  | "Battle Active"
  | "VICTORY"
  | "DEFEAT"
  | "Error";

interface TurnUpdate {
  user?: { row: number; col: number; result: CellState };
  bot?: { row: number; col: number; result: CellState };
  winner?: "User" | "Bot" | null;
}

interface WSMessage {
  type?: "init";
  status?: "success" | "error";
  board?: CellState[][];
  turn_update?: TurnUpdate;
  message?: string;
}

const GRID_SIZE = 10;
const WIN_THRESHOLD = 7;
const HTTP_URL =
  process.env.NEXT_PUBLIC_BACKEND_HTTP || "http://localhost:3000";
const WS_URL =
  process.env.NEXT_PUBLIC_BACKEND_WS || "ws://localhost:3000";

const createEmptyBoard = (): CellState[][] =>
  Array.from({ length: GRID_SIZE }, () => Array(GRID_SIZE).fill("Empty"));

export default function GamePage() {
  const [gameId, setGameId] = useState<string>("");
  const [status, setStatus] = useState<GameStatus>("Disconnected");
  const [winner, setWinner] = useState<string | null>(null);
  const [difficulty, setDifficulty] = useState<Difficulty>("Easy");

  const [myBoard, setMyBoard] = useState<CellState[][]>(createEmptyBoard);
  const [enemyBoard, setEnemyBoard] = useState<CellState[][]>(createEmptyBoard);

  const socketRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    return () => {
      if (socketRef.current) socketRef.current.close();
    };
  }, []);

  const updateGrid = (
    setBoard: React.Dispatch<React.SetStateAction<CellState[][]>>,
    row: number,
    col: number,
    result: CellState
  ) => {
    setBoard((prev) => {
      const newBoard = prev.map((r) => [...r]);
      newBoard[row][col] = result;
      return newBoard;
    });
  };

  const connectToWebsocket = useCallback((id: string) => {
    if (socketRef.current) return;

    const ws = new WebSocket(`${WS_URL}/ws/${id}`);

    ws.onopen = () => setStatus("Battle Active");

    ws.onmessage = (event) => {
      const data: WSMessage = JSON.parse(event.data);

      if (data.type === "init" && data.board) {
        setMyBoard(data.board);
      } else if (data.status === "success" && data.turn_update) {
        const { user, bot, winner: gameWinner } = data.turn_update;

        if (user) updateGrid(setEnemyBoard, user.row, user.col, user.result);
        if (bot) updateGrid(setMyBoard, bot.row, bot.col, bot.result);

        if (gameWinner) {
          setWinner(gameWinner);
          setStatus(gameWinner === "User" ? "VICTORY" : "DEFEAT");
          ws.close();
          socketRef.current = null;
        }
      } else if (data.status === "error") {
        console.warn("Maneuver Rejected:", data.message);
      }
    };

    socketRef.current = ws;
  }, []);

  const startBotGame = async () => {
    socketRef.current?.close();
    socketRef.current = null;
    setWinner(null);
    setGameId("");
    setMyBoard(createEmptyBoard());
    setEnemyBoard(createEmptyBoard());
    setStatus("Connecting...");

    try {
      const res = await fetch(`${HTTP_URL}/create_game`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ difficulty }),
      });

      if (!res.ok) throw new Error("Server Offline");

      const data = await res.json();
      setGameId(data.game_id);
      connectToWebsocket(data.game_id);
    } catch (err) {
      console.error(err);
      setStatus("Error");
    }
  };

  const handleShot = (row: number, col: number) => {
    if (!socketRef.current || winner || enemyBoard[row][col] !== "Empty") return;
    socketRef.current.send(`${row},${col}`);
  };

  return (
    <div className="flex flex-col items-center min-h-screen bg-zinc-950 text-zinc-100 font-sans px-4 py-8">
      <h1 className="text-3xl md:text-4xl mb-10 font-semibold tracking-[0.35em] uppercase text-zinc-100 border-b border-zinc-700 pb-4">
        Battleship Command
      </h1>

      {/* --- START SCREEN --- */}
      {!gameId && (
        <div className="flex flex-col items-center max-w-2xl w-full space-y-6">
          {status === "Connecting..." ? (
            <div className="text-sm text-zinc-400 font-medium mt-10">
              Establishing secure session…
            </div>
          ) : (
            <>
              {/* Mission card */}
              <div className="w-full bg-zinc-900 border border-zinc-700 rounded-3xl px-6 py-5 space-y-4">
                <h2 className="text-sm font-semibold tracking-[0.2em] text-zinc-200 uppercase">
                  Mission Briefing
                </h2>
                <ul className="space-y-2 text-sm text-zinc-300">
                  <li>
                    <span className="font-medium text-zinc-100">
                      Objective:
                    </span>{" "}
                    Eliminate the enemy fleet.
                  </li>
                  <li>
                    <span className="font-medium text-zinc-100">
                      The Fleet:
                    </span>{" "}
                    5 vessels (sizes 5, 4, 3, 3, 2).
                  </li>
                  <li>
                    <span className="font-medium text-zinc-100">
                      Victory:
                    </span>{" "}
                    First to land{" "}
                    <span className="font-semibold">{WIN_THRESHOLD} hits</span>.
                  </li>
                  <li className="text-xs text-zinc-500 italic pt-1">
                    High-speed simulation protocol active.
                  </li>
                </ul>
              </div>

              {/* Difficulty card */}
              <div className="w-full bg-zinc-900 border border-zinc-700 rounded-3xl px-6 py-4 space-y-4">
                <div className="flex items-center justify-between">
                  <span className="text-xs font-medium tracking-[0.2em] uppercase text-zinc-400">
                    Bot Difficulty
                  </span>
                  <span className="text-xs text-zinc-500">
                    Easy is recommended for first run
                  </span>
                </div>
                <div className="flex flex-wrap gap-3">
                  {(["Easy", "Medium", "Hard"] as Difficulty[]).map((lvl) => {
                    const isActive = difficulty === lvl;
                    return (
                      <button
                        key={lvl}
                        onClick={() => setDifficulty(lvl)}
                        className={`px-4 py-2 rounded-full text-xs font-medium border ${
                          isActive
                            ? "bg-zinc-100 text-zinc-900 border-zinc-100"
                            : "bg-zinc-900 text-zinc-400 border-zinc-600"
                        }`}
                      >
                        {lvl.toUpperCase()}
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* Start button */}
              <div className="w-full flex justify-center">
                <button
                  onClick={startBotGame}
                  className="px-8 py-3 bg-zinc-100 text-zinc-900 rounded-full text-sm font-semibold tracking-wide w-full md:w-auto"
                >
                  Start Mission
                </button>
              </div>
            </>
          )}
        </div>
      )}

      {gameId && (
        <div className="flex flex-col items-center gap-8 w-full max-w-6xl">
          {/* Top status bar */}
          <div className="flex items-center justify-between w-full bg-zinc-900 border border-zinc-700 rounded-2xl px-4 py-3">
            <div className="flex items-center gap-3">
              <div
                className={`px-3 py-1 rounded-full text-xs font-medium tracking-[0.15em] uppercase
                ${
                  winner === "User"
                    ? "bg-emerald-700/40 text-emerald-200"
                    : winner === "Bot"
                    ? "bg-rose-700/40 text-rose-200"
                    : "bg-zinc-800 text-zinc-300"
                }`}
              >
                {winner ? `${winner} Victory` : status}
              </div>
              <div className="hidden md:inline-flex px-3 py-1 rounded-full text-xs font-medium tracking-[0.15em] uppercase bg-zinc-900 border border-zinc-700 text-zinc-400">
                Bot Level: {difficulty}
              </div>
            </div>

            <button
              onClick={startBotGame}
              className="px-4 py-1.5 rounded-full border border-zinc-600 text-xs font-medium text-zinc-200 bg-zinc-900"
            >
              {winner ? "New Mission" : "Restart"}
            </button>
          </div>

          <div className="flex flex-col md:flex-row gap-10 justify-center w-full">
            {/* Player Sector */}
            <div className="flex flex-col items-center gap-3">
              <h2 className="text-xs font-semibold tracking-[0.2em] uppercase text-zinc-400">
                Your Fleet
              </h2>
              <div className="grid grid-cols-10 gap-1 bg-zinc-900 p-4 rounded-3xl border border-zinc-700">
                {myBoard.map((row, r) =>
                  row.map((cell, c) => (
                    <div
                      key={`my-${r}-${c}`}
                      className={`w-10 h-10 flex items-center justify-center rounded-md border text-[10px] ${
                        cell === "Ship"
                          ? "bg-indigo-600 border-indigo-600"
                          : cell === "Hit"
                          ? "bg-rose-500 border-rose-500 text-white"
                          : cell === "Miss"
                          ? "bg-zinc-700 border-zinc-700"
                          : "bg-zinc-900 border-zinc-700"
                      }`}
                    />
                  ))
                )}
              </div>
            </div>
            <div
              className={`flex flex-col items-center gap-3 ${
                winner ? "opacity-60" : ""
              }`}
            >
              <h2 className="text-xs font-semibold tracking-[0.2em] uppercase text-zinc-400">
                Enemy Grid
              </h2>
              <div className="grid grid-cols-10 gap-1 bg-zinc-900 p-4 rounded-3xl border border-zinc-700">
                {enemyBoard.map((row, r) =>
                  row.map((cell, c) => (
                    <div
                      key={`enemy-${r}-${c}`}
                      onClick={() => handleShot(r, c)}
                      className={`w-10 h-10 flex items-center justify-center rounded-md border text-[10px] ${
                        !winner && cell === "Empty"
                          ? "cursor-pointer bg-zinc-900 border-zinc-700"
                          : cell === "Hit"
                          ? "bg-rose-500 border-rose-500"
                          : cell === "Miss"
                          ? "bg-zinc-700 border-zinc-700"
                          : "bg-zinc-900 border-zinc-700"
                      }`}
                    />
                  ))
                )}
              </div>
              {!winner && (
                <div className="text-[12px] text-zinc-500 mt-1 tracking-[0.18em] uppercase">
                  Radar Active · Select Target
                </div>
              )}
            </div>
          </div>

          <div className="flex flex-wrap gap-6 text-[13px] text-zinc-400 mt-4 border-t border-zinc-700 pt-4 w-full justify-center">
            <div className="flex items-center gap-2">
              <span className="w-4 h-4 rounded-sm bg-indigo-600 border border-indigo-600" />
              Fleet (Your Ships)
            </div>
            <div className="flex items-center gap-2">
              <span className="w-4 h-4 rounded-sm bg-rose-500 border border-rose-500" />
              Hit
            </div>
            <div className="flex items-center gap-2">
              <span className="w-4 h-4 rounded-sm bg-zinc-700 border border-zinc-700" />
              Miss
            </div>
            <div className="flex items-center gap-2">
              <span className="w-4 h-4 rounded-sm bg-zinc-900 border border-zinc-700" />
              Unscanned
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
