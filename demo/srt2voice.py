#!/usr/bin/env python3
import argparse, os, subprocess, tempfile, math
import srt

def run(cmd: list[str]) -> None:
    p = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if p.returncode != 0:
        raise RuntimeError(f"Command failed:\n{' '.join(cmd)}\n\nSTDERR:\n{p.stderr}")
    return

def ffprobe_duration(path: str) -> float:
    cmd = [
        "ffprobe", "-v", "error",
        "-show_entries", "format=duration",
        "-of", "default=nokey=1:noprint_wrappers=1",
        path
    ]
    p = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if p.returncode != 0:
        raise RuntimeError(p.stderr)
    return float(p.stdout.strip())

def atempo_filter(rate: float) -> str:
    """
    ffmpeg atempo supports 0.5..2.0 per filter.
    For larger/smaller rates, chain multiple atempo filters.
    Here rate means: output_speed = rate (e.g., 1.1 = faster).
    """
    # clamp tiny values
    rate = max(rate, 1e-6)
    parts = []
    # bring rate into [0.5, 2] by factoring
    while rate > 2.0:
        parts.append("atempo=2.0")
        rate /= 2.0
    while rate < 0.5:
        parts.append("atempo=0.5")
        rate /= 0.5
    parts.append(f"atempo={rate:.6f}")
    return ",".join(parts)

def format_ts(td) -> float:
    return td.total_seconds()

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--srt", required=True, help="input .srt file")
    ap.add_argument("--out", default="voice.wav", help="output wav")
    ap.add_argument("--voice", default="zh-CN-YunxiNeural", help="Edge TTS voice name")
    ap.add_argument("--pad_ms", type=int, default=80, help="silence pad between segments (ms)")
    args = ap.parse_args()

    with open(args.srt, "r", encoding="utf-8") as f:
        subs = list(srt.parse(f.read()))

    if not subs:
        raise SystemExit("No subtitles found in SRT.")

    workdir = tempfile.mkdtemp(prefix="srt2voice_")
    seg_paths = []
    concat_list = os.path.join(workdir, "concat.txt")

    for i, sub in enumerate(subs, 1):
        text = sub.content.strip()
        if not text:
            continue

        start = format_ts(sub.start)
        end = format_ts(sub.end)
        slot = max(end - start, 0.1)

        raw_wav = os.path.join(workdir, f"seg{i:03d}_raw.wav")
        fit_wav = os.path.join(workdir, f"seg{i:03d}_fit.wav")

        # 1) TTS -> raw wav
        # edge-tts output supports wav directly
        run([
            "edge-tts",
            "--voice", args.voice,
            "--text", text,
            "--write-media", raw_wav
        ])

        raw_dur = ffprobe_duration(raw_wav)

        # 2) speed-fit:
        # if raw is longer than slot -> speed up (rate>1)
        # if raw is shorter -> slow down (rate<1)
        rate = raw_dur / slot  # >1 => need to speed up by this factor
        # but ffmpeg atempo expects speed factor, i.e. 1.25 means faster
        speed = rate

        # avoid insane stretching; if too extreme, better to trim/pad
        speed = min(max(speed, 0.6), 1.8)

        filt = atempo_filter(speed)

        # 3) Make exactly slot seconds:
        # - apply atempo
        # - then either trim or pad with silence to exact duration
        # - optional tiny inter-segment padding
        pad = args.pad_ms / 1000.0
        target = slot

        run([
            "ffmpeg", "-y", "-hide_banner", "-loglevel", "error",
            "-i", raw_wav,
            "-filter:a", f"{filt},atrim=0:{target:.6f},apad=pad_dur={target + pad:.6f},atrim=0:{target + pad:.6f}",
            "-ar", "48000", "-ac", "2",
            fit_wav
        ])

        seg_paths.append(fit_wav)

    if not seg_paths:
        raise SystemExit("No non-empty subtitle lines to synthesize.")

    # 4) concat all segments
    with open(concat_list, "w", encoding="utf-8") as f:
        for p in seg_paths:
            f.write(f"file '{p}'\n")

    run([
        "ffmpeg", "-y", "-hide_banner", "-loglevel", "error",
        "-f", "concat", "-safe", "0",
        "-i", concat_list,
        "-c", "pcm_s16le",
        args.out
    ])

    print(f"✅ Done: {args.out}")
    print(f"Workdir kept at: {workdir}")

if __name__ == "__main__":
    main()