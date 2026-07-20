#!/usr/bin/env python3
"""CLI wrapper for cross-session memory retrieval.

Usage:
    python3 .dijiang/scripts/memory.py search keyword <query>
    python3 .dijiang/scripts/memory.py search task <task-name>
    python3 .dijiang/scripts/memory.py search date --days 14
    python3 .dijiang/scripts/memory.py search platform pi
    python3 .dijiang/scripts/memory.py show <index>
"""

from __future__ import annotations

import sys
sys.path.insert(0, str(__import__("pathlib").Path(__file__).resolve().parent))

from common.memory import main

sys.exit(main() if __name__ == "__main__" else 0)
