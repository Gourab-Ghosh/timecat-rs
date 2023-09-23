#!/usr/bin/env python3

import itertools as it
import tempfile
import os, sys, shutil, time, random
from pprint import pprint
from datetime import datetime
from chess import Board, STARTING_BOARD_FEN, Outcome, Termination
from chess.engine import SimpleEngine, Limit
from engine_paths import FIRST_ENGINE_PATH, SECOND_ENGINE_PATH

print(f"Engine 1 path: {FIRST_ENGINE_PATH}")
print(f"Engine 2 path: {SECOND_ENGINE_PATH}")

LOG_FILE_PATH = __file__.replace(".py", ".log")
OVERWRITE_LOG_FILE = True
# LOG_FILE_PATH = __file__.replace(".py", ".log").replace(".log", "1.log")
ENGINE_NAME_LENGTH = 15
LIMIT = Limit(time = 1/10)

class Engine(SimpleEngine):

    def clear_hash():
        pass

def play_game(engine1: Engine, engine2: Engine, limit, fen = STARTING_BOARD_FEN, print_info = True, af_path = None) -> Outcome:
    board = Board(fen)
    engine1.clear_hash()
    engine2.clear_hash()
    if engine1 is not engine2:
        engine1.index = 1
        engine2.index = 2
    engine_iterator = it.cycle([engine1, engine2])
    while not (board.is_game_over() or board.is_fifty_moves() or board.is_repetition(3)):
        engine = next(engine_iterator)
        start_time = time.time()
        analysis = engine.analyse(board, limit)
        time_taken = time.time() - start_time
        if limit.time is not None:
            if time_taken > max(3 * limit.time, 0.1):
                print(f"Engine {engine.index} exceeded time limit of {limit.time} s by {round(time_taken - limit.time, 3)} s with fen {board.fen()}")
        move = analysis["pv"][0]
        if print_info:
            if engine1 is engine2:
                print(f"Engine played {board.san(move)} in {round(time_taken, 3)} s")
            else:
                print(f"Engine {board.san(move)} played {engine.index} in {round(time_taken, 3)} s")
        board.push(move)
    pgn = Board(fen).variation_san(board.move_stack.copy())
    if print_info:
        print(pgn)
    outcome = board.outcome()
    if outcome is None:
        if board.is_repetition(3):
            outcome = Outcome(Termination.THREEFOLD_REPETITION, None)
        elif board.is_fifty_moves():
            outcome = Outcome(Termination.FIFTY_MOVES, None)
    result = outcome.result() if outcome else "1/2-1/2"
    if af_path is not None:
        with open(af_path, "a") as af:
            af.write(f"fen: {fen}, limit: {limit}\n{pgn} {result}\n\n")
    return outcome

def get_fen_and_opening_name(line):
    fen, opening = line.split("(", 1)
    fen = fen.strip()
    opening = opening.strip().rstrip(")")
    return fen, opening

def get_stats_string(stats):
    return f"Engine1 Win: {stats[True]}, Engine2 Win: {stats[False]}, Draw: {stats[None]}"

def get_engine_paths():
    engine_paths = []
    for engine_path in [FIRST_ENGINE_PATH, SECOND_ENGINE_PATH]:
        if not os.path.isfile(engine_path):
            raise FileNotFoundError(f"Engine path {engine_path} not found")
        temp_dir = os.path.join(tempfile.gettempdir(), "engine_binaries")
        os.makedirs(temp_dir, exist_ok = True)
        _, engine_ext = os.path.splitext(engine_path)
        start_num = 1 << (4 * (ENGINE_NAME_LENGTH - 1))
        end_num = (start_num << 4) - 1
        engine_new_path = os.path.join(temp_dir, hex(random.randint(start_num, end_num))[2:].upper() + engine_ext)
        shutil.copy(engine_path, engine_new_path)
        engine_paths.append(engine_new_path)
    return engine_paths

def main():
    engine_paths = get_engine_paths()
    random.seed(0)
    with Engine.popen_uci(engine_paths[0]) as engine1, Engine.popen_uci(engine_paths[1]) as engine2:
        if "Hash" in engine1.options.keys() and "Hash" in engine2.options.keys():
            engine1.configure({"Hash": 64})
            engine2.configure({"Hash": 64})
        with open("test_fens.txt", "r") as rf:
            fens_and_opening_name = [get_fen_and_opening_name(line) for line in rf.readlines()]
        random.shuffle(fens_and_opening_name)
        stats = {
            True: 0,
            False: 0,
            None: 0,
        }
        total_time = 0
        if OVERWRITE_LOG_FILE and os.path.isfile(LOG_FILE_PATH):
            os.remove(LOG_FILE_PATH)
        print(f"Playing {len(fens_and_opening_name)} games with limit {LIMIT}")
        for i, (fen, opening) in enumerate(fens_and_opening_name):
            try:
                start_time = time.time()
                outcome = play_game(engine1, engine2, LIMIT, fen, "--verbose" in sys.argv, LOG_FILE_PATH)
                time_taken = time.time() - start_time
                total_time += time_taken
                stats[outcome.winner] += 1
                winner = "White" if outcome.winner else "None" if outcome.winner is None else "Black"
                time_left = datetime.utcfromtimestamp(total_time / (i + 1) * (len(fens_and_opening_name) - i - 1)).time()
                print(f"Game {i + 1} played in {round(time_taken, 3)} sec with opening {opening}, winner is {winner}, expected time left to finish: {time_left}")
                print(f"Stats till now: {get_stats_string(stats)}")
            except KeyboardInterrupt:
                break
        print(f"Stats: {get_stats_string(stats)}")
        print(f"Total time taken: {datetime.utcfromtimestamp(total_time).time()}")

if __name__ == "__main__":
    main()