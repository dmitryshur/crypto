import type { Request, Response, Express, RequestHandler, NextFunction } from 'express';
const express = require('express');
const proxy = require('express-http-proxy');

const CACHE_CLEAR_TIMEOUT = 30000;

interface ProxyCache {
  [key: string]: any;
}

class Proxy {
  readonly #domain: string;
  #cache: ProxyCache = {};

  constructor(domain: string) {
    this.#domain = domain;
  }

  start(port: number): void {
    const app: Express = express();

    app.use(this.createCacheMiddleware());
    app.use(this.createProxyMiddleware());

    app.listen(port, () => {
      console.log(`Listening on port ${port}`);
    });
  }

  private createCacheMiddleware(): RequestHandler {
    return ({ originalUrl }: Request, response: Response, next: NextFunction) => {
      if (this.#cache[originalUrl]) {
        console.log(`Serving cache for url ${originalUrl}`);
        response.header('X-Cached', 'true');
        response.send(this.#cache[originalUrl]);
        return;
      }

      next();
    };
  }

  private createProxyMiddleware(): RequestHandler {
    return proxy(this.#domain, {
      timeout: 5000,
      userResDecorator: (_proxyResonse: Response, proxyResponseData: any, userRequest: Request) => {
        const data = JSON.parse(proxyResponseData.toString('utf8'));
        const url = userRequest.originalUrl;

        console.log(`Caching data for url ${url}`);
        this.#cache[url] = data;
        setTimeout(this.clearCache(url), CACHE_CLEAR_TIMEOUT);

        return JSON.stringify(data);
      },
    });
  }

  private clearCache(key: string): TimerHandler {
    return () => {
      this.#cache[key] = null;
    };
  }
}

(function main() {
  const port = Number(process.env.PROXY_PORT) || 3000;
  const proxy = new Proxy('https://api.kraken.com');

  proxy.start(port);
})();
