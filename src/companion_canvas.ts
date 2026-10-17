// Aether Core 2D Pixel-Art Canvas Layered Engine

export type CompanionState = "idle" | "listening" | "throwing" | "muted" | "afk";

export interface CompanionCosmetic {
  id: string;
  name: string;
  description: string;
  category: "head" | "eyes" | "aura";
  unlockedAtWords: number;
}

export const COSMETIC_CATALOG: CompanionCosmetic[] = [
  {
    id: "default_halo",
    name: "Aether Rings",
    description: "Classic dual-band isometric orbital rings glowing with #00F2FE cyan energy.",
    category: "aura",
    unlockedAtWords: 0,
  },
  {
    id: "cyber_headphones",
    name: "Cyber Over-Ear",
    description: "Studio monitors with active audio visualizer LED pulses on each earcup.",
    category: "head",
    unlockedAtWords: 100,
  },
  {
    id: "neon_visor",
    name: "Neon Visor",
    description: "Cyberpunk glowing indigo eyewear with retro horizontal scanline accents.",
    category: "eyes",
    unlockedAtWords: 500,
  },
  {
    id: "golden_crown",
    name: "Golden Crown",
    description: "Royal 8-bit golden crown crowned with a radiant ruby centerpiece.",
    category: "head",
    unlockedAtWords: 1000,
  },
  {
    id: "star_aura",
    name: "Cosmic Starfield",
    description: "Orbiting celestial stars and nebula stardust swirling in perpetual motion.",
    category: "aura",
    unlockedAtWords: 5000,
  },
];

interface Projectile {
  x: number;
  y: number;
  vx: number;
  vy: number;
  radius: number;
  alpha: number;
  life: number;
  maxLife: number;
}

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  size: number;
  alpha: number;
  life: number;
  maxLife: number;
  color: string;
}

interface SnoozeZ {
  x: number;
  y: number;
  size: number;
  alpha: number;
  speed: number;
}

export class AetherCoreRenderer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private frames: Map<string, HTMLImageElement> = new Map();

  public state: CompanionState = "idle";
  public amplitude: number = 0; // 0.0 to 1.0 (audio dB bounce)
  public equippedCosmetics: Set<string> = new Set(["default_halo"]);
  public scale: number = 1.0;

  private animFrameId: number | null = null;
  private lastTime: number = performance.now();

  private throwTimer: number = 0;
  private projectiles: Projectile[] = [];
  private ambientParticles: Particle[] = [];
  private snoozeZs: SnoozeZ[] = [];
  private lastBlinkTime: number = 0;
  private isBlinking: boolean = false;
  private pokeBounce: number = 0;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    const context = canvas.getContext("2d");
    if (!context) throw new Error("Could not acquire 2D canvas context");
    this.ctx = context;

    this.preloadFrames();
    this.initParticles();
  }

  private preloadFrames() {
    const frameNames = [
      "frame_01",
      "frame_02",
      "frame_03",
      "frame_06",
      "frame_07",
      "frame_08",
    ];

    frameNames.forEach((name) => {
      const img = new Image();
      img.src = `/src/assets/companion/${name}.png`;
      this.frames.set(name, img);
    });
  }

  private initParticles() {
    this.ambientParticles = [];
    for (let i = 0; i < 16; i++) {
      this.ambientParticles.push({
        x: Math.random() * 200,
        y: Math.random() * 200,
        vx: (Math.random() - 0.5) * 0.4,
        vy: -0.2 - Math.random() * 0.3,
        size: 1 + Math.random() * 2,
        alpha: 0.2 + Math.random() * 0.6,
        life: Math.random() * 100,
        maxLife: 80 + Math.random() * 60,
        color: Math.random() > 0.5 ? "#00F2FE" : "#4FACFE",
      });
    }
  }

  public poke() {
    this.pokeBounce = 1.0;
  }

  public triggerThrow() {
    this.state = "throwing";
    this.throwTimer = 1.0;

    // Emit 3 sonic rings outward
    for (let i = 0; i < 3; i++) {
      const angle = -0.3 + (i - 1) * 0.25;
      const speed = 3.5 + i * 1.2;
      this.projectiles.push({
        x: this.canvas.width / 2,
        y: this.canvas.height / 2 - 10,
        vx: Math.cos(angle) * speed,
        vy: Math.sin(angle) * speed,
        radius: 12 + i * 6,
        alpha: 1.0,
        life: 0,
        maxLife: 35 + i * 8,
      });
    }
  }

  public start() {
    if (this.animFrameId !== null) return;
    const loop = (now: number) => {
      this.update(now);
      this.render(now);
      this.animFrameId = requestAnimationFrame(loop);
    };
    this.animFrameId = requestAnimationFrame(loop);
  }

  public stop() {
    if (this.animFrameId !== null) {
      cancelAnimationFrame(this.animFrameId);
      this.animFrameId = null;
    }
  }

  private update(now: number) {
    const dt = Math.min((now - this.lastTime) / 1000, 0.1);
    this.lastTime = now;

    // Decay poke bounce
    if (this.pokeBounce > 0) {
      this.pokeBounce = Math.max(0, this.pokeBounce - dt * 2.5);
    }

    // Handle throw state decay
    if (this.state === "throwing") {
      this.throwTimer -= dt * 1.5;
      if (this.throwTimer <= 0) {
        this.state = "idle";
      }
    }

    // Update Projectiles
    for (let i = this.projectiles.length - 1; i >= 0; i--) {
      const p = this.projectiles[i];
      p.x += p.vx;
      p.y += p.vy;
      p.radius += 0.8;
      p.life++;
      p.alpha = Math.max(0, 1.0 - p.life / p.maxLife);
      if (p.life >= p.maxLife) {
        this.projectiles.splice(i, 1);
      }
    }

    // Update Ambient Particles
    const w = this.canvas.width;
    const h = this.canvas.height;
    const speedMultiplier = this.state === "listening" ? 2.5 : 1.0;

    this.ambientParticles.forEach((p) => {
      p.x += p.vx * speedMultiplier;
      p.y += p.vy * speedMultiplier;
      p.life += 1;
      if (p.life >= p.maxLife || p.y < -10 || p.x < -10 || p.x > w + 10) {
        p.x = w / 2 + (Math.random() - 0.5) * 80;
        p.y = h / 2 + 30 + Math.random() * 20;
        p.vx = (Math.random() - 0.5) * 0.6;
        p.vy = -0.3 - Math.random() * 0.5;
        p.life = 0;
      }
    });

    // Update AFK Snooze Zs
    if (this.state === "afk") {
      if (Math.random() < 0.03 && this.snoozeZs.length < 5) {
        this.snoozeZs.push({
          x: w / 2 + 15,
          y: h / 2 - 20,
          size: 10 + Math.random() * 4,
          alpha: 1.0,
          speed: 0.5 + Math.random() * 0.4,
        });
      }
      for (let i = this.snoozeZs.length - 1; i >= 0; i--) {
        const z = this.snoozeZs[i];
        z.y -= z.speed;
        z.x += Math.sin(now * 0.003 + i) * 0.4;
        z.alpha -= 0.008;
        if (z.alpha <= 0 || z.y < 0) {
          this.snoozeZs.splice(i, 1);
        }
      }
    } else {
      this.snoozeZs = [];
    }

    // Blink timer (every 3.5 - 5 seconds)
    if (now - this.lastBlinkTime > 3800) {
      this.isBlinking = true;
      if (now - this.lastBlinkTime > 4100) {
        this.isBlinking = false;
        this.lastBlinkTime = now;
      }
    }
  }

  private render(now: number) {
    const ctx = this.ctx;
    const w = this.canvas.width;
    const h = this.canvas.height;

    ctx.clearRect(0, 0, w, h);
    ctx.imageSmoothingEnabled = false;

    const cx = w / 2;
    const cy = h / 2;

    // Calculate vertical hover displacement (4px sinusoidal)
    const hoverSpeed = this.state === "afk" ? 0.0012 : this.state === "listening" ? 0.0045 : 0.0025;
    const hoverY = Math.sin(now * hoverSpeed) * (this.state === "afk" ? 2.5 : 4.5);

    // Audio amplitude bounce (scales body up when speaking)
    const ampScale = this.state === "listening" ? 1.0 + this.amplitude * 0.18 : 1.0;
    const pokeScale = 1.0 + Math.sin(this.pokeBounce * Math.PI) * 0.15;
    const totalScale = this.scale * ampScale * pokeScale;

    // 1. Render Floor Shadow
    this.renderFloorShadow(ctx, cx, cy + 52, hoverY, totalScale);

    // 2. Render Ambient VFX & Particles (Behind core)
    this.renderAmbientParticles(ctx);

    // 3. Render Star Aura (if equipped)
    if (this.equippedCosmetics.has("star_aura")) {
      this.renderStarAura(ctx, cx, cy + hoverY, now);
    }

    // 4. Render Core Character Sprite
    ctx.save();
    ctx.translate(cx, cy + hoverY);
    ctx.scale(totalScale, totalScale);

    // Glow in listening mode
    if (this.state === "listening") {
      ctx.shadowColor = "#00F2FE";
      ctx.shadowBlur = 18 + this.amplitude * 20;
    } else if (this.state === "throwing") {
      ctx.shadowColor = "#FFFFFF";
      ctx.shadowBlur = 24;
    }

    this.renderActiveFrame(ctx, now);

    // 5. Render Wearable Accessories
    this.renderCosmetics(ctx, now);

    ctx.restore();

    // 6. Render Action VFX (Projectiles & Sonic Rings)
    this.renderProjectiles(ctx);

    // 7. Render AFK Snooze Zs
    if (this.state === "afk") {
      this.renderSnoozeZs(ctx);
    }
  }

  private renderFloorShadow(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    hoverY: number,
    scale: number
  ) {
    ctx.save();
    const shadowScale = (1.0 - hoverY * 0.04) * scale;
    ctx.fillStyle = "rgba(10, 16, 28, 0.45)";
    ctx.beginPath();
    ctx.ellipse(x, y, 42 * shadowScale, 9 * shadowScale, 0, 0, Math.PI * 2);
    ctx.fill();
    ctx.restore();
  }

  private renderAmbientParticles(ctx: CanvasRenderingContext2D) {
    ctx.save();
    this.ambientParticles.forEach((p) => {
      ctx.fillStyle = p.color;
      ctx.globalAlpha = p.alpha;
      ctx.fillRect(Math.round(p.x), Math.round(p.y), p.size, p.size);
    });
    ctx.restore();
  }

  private renderActiveFrame(ctx: CanvasRenderingContext2D, now: number) {
    let frameKey = "frame_01";

    if (this.state === "listening") {
      frameKey = "frame_08";
    } else if (this.state === "throwing") {
      frameKey = "frame_08";
    } else if (this.state === "muted") {
      frameKey = "frame_06";
    } else if (this.state === "afk") {
      frameKey = Math.sin(now * 0.001) > 0 ? "frame_06" : "frame_07";
    } else {
      if (this.isBlinking) {
        frameKey = "frame_03";
      } else {
        const glanceCycle = Math.floor(now / 2000) % 5;
        if (glanceCycle === 2) {
          frameKey = "frame_02";
        } else {
          frameKey = "frame_01";
        }
      }
    }

    const img = this.frames.get(frameKey);
    if (img && img.complete) {
      const size = 130;
      ctx.drawImage(img, -size / 2, -size / 2, size, size);
    } else {
      ctx.fillStyle = "#00F2FE";
      ctx.beginPath();
      ctx.arc(0, 0, 36, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  private renderCosmetics(ctx: CanvasRenderingContext2D, now: number) {
    if (this.equippedCosmetics.has("cyber_headphones")) {
      this.drawHeadphones(ctx, now);
    }
    if (this.equippedCosmetics.has("golden_crown")) {
      this.drawCrown(ctx, now);
    }
    if (this.equippedCosmetics.has("neon_visor")) {
      this.drawVisor(ctx, now);
    }
  }

  private drawHeadphones(ctx: CanvasRenderingContext2D, now: number) {
    ctx.save();
    ctx.strokeStyle = "#1A2236";
    ctx.lineWidth = 5;
    ctx.beginPath();
    ctx.arc(0, -6, 38, Math.PI * 1.15, Math.PI * 1.85);
    ctx.stroke();

    ctx.strokeStyle = "#00F2FE";
    ctx.lineWidth = 2;
    ctx.stroke();

    ctx.fillStyle = "#0E1524";
    ctx.strokeStyle = "#4FACFE";
    ctx.lineWidth = 1.5;
    ctx.fillRect(-41, -16, 8, 24);
    ctx.strokeRect(-41, -16, 8, 24);

    ctx.fillRect(33, -16, 8, 24);
    ctx.strokeRect(33, -16, 8, 24);

    const glowH = 4 + (this.state === "listening" ? this.amplitude * 12 : Math.sin(now * 0.005) * 4);
    ctx.fillStyle = "#00F2FE";
    ctx.fillRect(-39, -4 - glowH / 2, 4, glowH);
    ctx.fillRect(35, -4 - glowH / 2, 4, glowH);

    ctx.restore();
  }

  private drawCrown(ctx: CanvasRenderingContext2D, now: number) {
    ctx.save();
    const cx = 0;
    const cy = -42;

    ctx.fillStyle = "#FFD700";
    ctx.beginPath();
    ctx.moveTo(cx - 18, cy);
    ctx.lineTo(cx - 20, cy - 14);
    ctx.lineTo(cx - 10, cy - 8);
    ctx.lineTo(cx, cy - 18);
    ctx.lineTo(cx + 10, cy - 8);
    ctx.lineTo(cx + 20, cy - 14);
    ctx.lineTo(cx + 18, cy);
    ctx.closePath();
    ctx.fill();

    ctx.strokeStyle = "#FFA500";
    ctx.lineWidth = 1.5;
    ctx.stroke();

    ctx.fillStyle = "#FF1E56";
    ctx.beginPath();
    ctx.arc(cx, cy - 6, 3, 0, Math.PI * 2);
    ctx.fill();

    if (Math.sin(now * 0.003) > 0.7) {
      ctx.fillStyle = "#FFFFFF";
      ctx.fillRect(cx - 1, cy - 19, 2, 2);
    }
    ctx.restore();
  }

  private drawVisor(ctx: CanvasRenderingContext2D, now: number) {
    ctx.save();
    const cy = -3;
    ctx.fillStyle = "rgba(79, 172, 254, 0.85)";
    ctx.strokeStyle = "#00F2FE";
    ctx.lineWidth = 1.5;

    ctx.beginPath();
    ctx.roundRect(-24, cy - 6, 48, 13, 3);
    ctx.fill();
    ctx.stroke();

    ctx.fillStyle = "rgba(255, 255, 255, 0.7)";
    const glareX = -18 + ((now * 0.04) % 36);
    ctx.fillRect(glareX, cy - 4, 6, 2);

    ctx.restore();
  }

  private renderStarAura(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    now: number
  ) {
    ctx.save();
    ctx.translate(x, y);

    const stars = 5;
    for (let i = 0; i < stars; i++) {
      const angle = (now * 0.0018 + (i * Math.PI * 2) / stars) % (Math.PI * 2);
      const rx = 58;
      const ry = 22;
      const sx = Math.cos(angle) * rx;
      const sy = Math.sin(angle) * ry;

      ctx.fillStyle = i % 2 === 0 ? "#00F2FE" : "#FFFFFF";
      ctx.shadowColor = "#00F2FE";
      ctx.shadowBlur = 8;

      ctx.fillRect(sx - 1, sy - 3, 2, 6);
      ctx.fillRect(sx - 3, sy - 1, 6, 2);
    }
    ctx.restore();
  }

  private renderProjectiles(ctx: CanvasRenderingContext2D) {
    ctx.save();
    this.projectiles.forEach((p) => {
      ctx.strokeStyle = `rgba(0, 242, 254, ${p.alpha})`;
      ctx.lineWidth = 2.5;
      ctx.shadowColor = "#00F2FE";
      ctx.shadowBlur = 12;

      ctx.beginPath();
      ctx.ellipse(p.x, p.y, p.radius, p.radius * 0.45, -0.2, 0, Math.PI * 2);
      ctx.stroke();

      ctx.strokeStyle = `rgba(255, 255, 255, ${p.alpha * 0.8})`;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.ellipse(p.x, p.y, p.radius * 0.6, p.radius * 0.28, -0.2, 0, Math.PI * 2);
      ctx.stroke();
    });
    ctx.restore();
  }

  private renderSnoozeZs(ctx: CanvasRenderingContext2D) {
    ctx.save();
    this.snoozeZs.forEach((z) => {
      ctx.fillStyle = `rgba(0, 242, 254, ${z.alpha})`;
      ctx.font = `bold ${Math.round(z.size)}px monospace`;
      ctx.shadowColor = "#4FACFE";
      ctx.shadowBlur = 6;
      ctx.fillText("z", z.x, z.y);
    });
    ctx.restore();
  }
}
