#!/usr/bin/env python3
"""CLI wrapper for spec registry sync.

Usage:
    python3 .dijiang/scripts/spec_sync.py check          # Check for drift
    python3 .dijiang/scripts/spec_sync.py update         # Update registry
    python3 .dijiang/scripts/spec_sync.py status         # Show status
"""

from __future__ import annotations

import sys
sys.path.insert(0, str(__import__("pathlib").Path(__file__).resolve().parent))

from common.spec_sync import main

sys.exit(main() if __name__ == "__main__" else 0)
