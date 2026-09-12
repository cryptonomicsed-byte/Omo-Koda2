/**
 * Swibe Witness Engine — Multimodal Perception
 *
 * Processes images, video frames, and audio to produce
 * unified embeddings for the memory engine.
 */

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

class WitnessEngine {
  constructor(config = {}) {
    this.config = {
      fusion: config.fusion || 'unified_context',
      maxFrames: config.maxFrames || 3,
      ...config,
    };
    this.observations = [];
  }

  async perceive(inputs) {
    const results = {};

    if (inputs.image) {
      results.image = await this.analyzeImage(inputs.image);
    }

    if (inputs.video) {
      results.video = await this.analyzeVideo(
        inputs.video,
        inputs.extract_frames || this.config.maxFrames
      );
    }

    if (inputs.audio) {
      results.audio = await this.analyzeAudio(inputs.audio, {
        transcribe: inputs.transcribe !== false,
        emotionDetect: inputs.emotion_detect || false,
      });
    }

    const fused = this.fuse(results);
    this.observations.push({
      timestamp: Date.now(),
      inputs: Object.keys(inputs),
      hash: crypto.createHash('sha256')
        .update(JSON.stringify(fused))
        .digest('hex')
        .slice(0, 16),
      result: fused,
    });

    return fused;
  }

  async analyzeImage(imagePath) {
    const resolved = path.resolve(imagePath);
    if (!fs.existsSync(resolved)) {
      return { error: `Image not found: ${resolved}`, modality: 'image' };
    }
    const stat = fs.statSync(resolved);
    const ext = path.extname(resolved).toLowerCase();
    return {
      modality: 'image',
      path: resolved,
      format: ext,
      size: stat.size,
      analyzed: true,
      description: `[image:${path.basename(resolved)}]`,
      embedding: this._mockEmbedding(resolved),
    };
  }

  async analyzeVideo(videoPath, frameCount) {
    const resolved = path.resolve(videoPath);
    if (!fs.existsSync(resolved)) {
      return { error: `Video not found: ${resolved}`, modality: 'video' };
    }
    const frames = [];
    for (let i = 0; i < frameCount; i++) {
      frames.push({
        index: i,
        timestamp_s: i * 2,
        description: `[frame:${i}]`,
        embedding: this._mockEmbedding(`${resolved}:frame${i}`),
      });
    }
    return { modality: 'video', path: resolved, frames, analyzed: true };
  }

  async analyzeAudio(audioPath, options = {}) {
    const resolved = path.resolve(audioPath);
    if (!fs.existsSync(resolved)) {
      return { error: `Audio not found: ${resolved}`, modality: 'audio' };
    }
    const result = { modality: 'audio', path: resolved, analyzed: true };
    if (options.transcribe) {
      result.transcript = `[transcript:${path.basename(resolved)}]`;
    }
    if (options.emotionDetect) {
      result.emotion = { valence: 0.6, arousal: 0.4, label: 'neutral' };
    }
    result.embedding = this._mockEmbedding(resolved);
    return result;
  }

  fuse(results) {
    const modalities = Object.keys(results);
    const embeddings = modalities
      .map(m => results[m]?.embedding)
      .filter(Boolean);
    return {
      modalities,
      strategy: this.config.fusion,
      results,
      unified_embedding: embeddings.length > 0
        ? this._averageEmbeddings(embeddings)
        : null,
      timestamp: Date.now(),
    };
  }

  _mockEmbedding(seed) {
    const hash = crypto.createHash('sha256').update(seed).digest();
    const emb = [];
    for (let i = 0; i < 8; i++) {
      emb.push(((hash[i] || 0) / 255) * 2 - 1);
    }
    return emb;
  }

  _averageEmbeddings(embeddings) {
    if (embeddings.length === 0) return [];
    const dims = embeddings[0].length;
    const avg = new Array(dims).fill(0);
    for (const emb of embeddings) {
      for (let i = 0; i < dims; i++) {
        avg[i] += emb[i] / embeddings.length;
      }
    }
    return avg;
  }

  getHistory() {
    return this.observations;
  }
}

export { WitnessEngine };
