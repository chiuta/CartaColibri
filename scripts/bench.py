#!/usr/bin/env python3
"""Scale benchmark: builds a synthetic library (Zipf vocabulary, a few duplicated and dated documents) and times indexing,
searches and the whole-library analyses with the real binary.
Usage: python3 scripts/bench.py [--docs 50000] [--bin target/release/cartacolibri] [--keep]"""
import argparse, os, random, shutil, subprocess, sys, tempfile, time, json, resource
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def main():
    ap = argparse.ArgumentParser(); ap.add_argument("--docs", type=int, default=50000); ap.add_argument("--bin", default=os.path.join(ROOT, "target", "release", "cartacolibri")); ap.add_argument("--keep", action="store_true"); a = ap.parse_args()
    rnd = random.Random(7); vocab = [f"w{i:05d}x" for i in range(20000)] + "payment invoice contract delivery supplier penalty warehouse lease rent notice term".split()
    weights = [1 / (i + 1) for i in range(len(vocab))]
    tmp = tempfile.mkdtemp(prefix="cc-bench-"); docs = os.path.join(tmp, "docs"); home = os.path.join(tmp, "home"); os.makedirs(home)
    t = time.time()
    for i in range(a.docs):
        d = os.path.join(docs, f"d{i // 500:04d}"); os.makedirs(d, exist_ok=True)
        words = rnd.choices(vocab, weights, k=rnd.randint(80, 300))
        body = " ".join(words)
        if i % 97 == 0: body += f" Signed on {rnd.randint(1,28):02d}.{rnd.randint(1,12):02d}.{rnd.randint(2018,2025)}. Rent is {rnd.randint(1000,9000)} lei per month."
        open(os.path.join(d, f"f{i}.txt"), "w").write(body)
        if i % 400 == 0: open(os.path.join(d, f"f{i}-copy.txt"), "w").write(body)
    size = sum(os.path.getsize(os.path.join(r, f)) for r, _, fs in os.walk(docs) for f in fs)
    print(f"generated {a.docs} docs, {size/1e6:.0f} MB text in {time.time()-t:.0f}s")
    def run(args, label, timeout=3600):
        t = time.time(); r = subprocess.run([a.bin, "--home", home] + args, capture_output=True, text=True, timeout=timeout); el = time.time() - t
        ru = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss / 1024
        print(f"{label:38} {el:8.2f}s   rc={r.returncode}   maxrss(so far)={ru:.0f} MB"); return r, el
    run(["add", docs], "add + first index")
    r, _ = run(["index"], "re-index (nothing changed)")
    open(os.path.join(docs, "d0000", "new.txt"), "w").write("a brand new document about warehouse penalty")
    run(["index"], "re-index (1 new file)")
    print("index size: %.0f MB" % (os.path.getsize(os.path.join(home, "index.sqlite")) / 1e6))
    for q, lbl in [("payment", "search: common word"), ("w00042x", "search: rare word"), ('"payment invoice"', "search: phrase"), ("pay*", "search: prefix"), ("payment AND NOT invoice", "search: boolean"), ("re:w0001\\dx", "search: regex"), ("payment ext:txt after:2020", "search: with filters"), ("payment path:d0007", "search: path filter")]:
        run(["search", q, "--json"], lbl)
    run(["stats"], "stats"); run(["timeline"], "timeline (documents by month)")
    run(["entities"], "entities (whole library)", 1800)
    run(["chronology", "--limit", "50"], "chronology", 1800); run(["dupes"], "dupes (whole library)", 1800)
    run(["conflicts"], "conflicts (whole library)", 1800); run(["topics"], "topics (whole library)", 1800)
    run(["dossier", "warehouse", "--json"], "dossier")
    if not a.keep: shutil.rmtree(tmp, ignore_errors=True)
    else: print("kept", tmp)

if __name__ == "__main__": main()
