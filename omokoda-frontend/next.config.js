/** @type {import('next').NextConfig} */
const KERNEL = process.env.OMOKODA_URL || 'http://127.0.0.1:7777'

const nextConfig = {
  typescript: {
    ignoreBuildErrors: false,
  },
  eslint: {
    ignoreDuringBuilds: false,
  },
  // Proxy the kernel API so client-side relative `/v1/*` calls (e.g. the Memory
  // Vault tab) actually reach the Omo-Koda kernel instead of 404-ing on the
  // frontend origin. Override the target with OMOKODA_URL.
  async rewrites() {
    return [
      { source: '/v1/:path*', destination: `${KERNEL}/v1/:path*` },
    ]
  },
}

module.exports = nextConfig
