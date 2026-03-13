import type {
  Chain,
  DecodeErrorEnvelope,
  DecodeSuccessEnvelope,
  ExamplesResponse,
  HealthResponse,
  IndexedDecodeResponse,
  IndexedListResponse,
  NormalizedTransaction,
} from "./types.js";

export interface ChainMergeClientOptions {
  /**
   * Base URL of the ChainMerge API, e.g.:
   * - "http://127.0.0.1:8080" for local development
   * - "https://api.chainmerge.io" for a hosted deployment
   *
   * Do not include a trailing slash.
   */
  baseUrl: string;

  /**
   * Optional API key that will be sent as "x-api-key".
   */
  apiKey?: string;

  /**
   * Custom fetch implementation for environments where "fetch"
   * is not globally available (e.g. Node < 18).
   */
  fetchImpl?: typeof fetch;
}

export class ChainMergeClient {
  private readonly baseUrl: string;
  private readonly apiKey?: string;
  private readonly fetchImpl: typeof fetch;

  constructor(options: ChainMergeClientOptions) {
    if (!options.baseUrl) {
      throw new Error("ChainMergeClient: baseUrl is required");
    }

    this.baseUrl = options.baseUrl.replace(/\/+$/, "");
    this.apiKey = options.apiKey;
    this.fetchImpl = options.fetchImpl ?? fetch;
  }

  /**
   * Check API health.
   */
  async health(): Promise<HealthResponse> {
    return this.get<HealthResponse>("/api/health");
  }

  /**
   * Get example transactions supported by the API.
   */
  async examples(): Promise<ExamplesResponse> {
    return this.get<ExamplesResponse>("/api/examples");
  }

  /**
   * Get internal API metrics (Prometheus format string).
   */
  async metrics(): Promise<string> {
    const url = `${this.baseUrl}/api/metrics`;
    const res = await this.fetchImpl(url, {
      headers: this.getHeaders(),
    });
    if (!res.ok) {
      throw await this.handleError(res);
    }
    return res.text();
  }

  /**
   * Decode a single transaction into a normalized representation.
   */
  async decodeTx(params: {
    chain: Chain;
    hash: string;
    rpcUrl?: string;
  }): Promise<NormalizedTransaction> {
    const search = this.buildSearchParams(params);
    const url = `${this.baseUrl}/api/decode?${search.toString()}`;
    const res = await this.fetchImpl(url, {
      headers: this.getHeaders(),
    });

    if (!res.ok) {
      throw await this.handleError(res);
    }

    const body = (await res.json()) as DecodeSuccessEnvelope;
    return body.decoded;
  }

  /**
   * Decode a transaction and persist it in the backend index.
   */
  async decodeAndIndexTx(params: {
    chain: Chain;
    hash: string;
    rpcUrl?: string;
  }): Promise<NormalizedTransaction> {
    const search = this.buildSearchParams(params);
    const url = `${this.baseUrl}/api/index/decode?${search.toString()}`;
    const res = await this.fetchImpl(url, {
      headers: this.getHeaders(),
    });

    if (!res.ok) {
      throw await this.handleError(res);
    }

    const body = (await res.json()) as IndexedDecodeResponse;
    return body.decoded;
  }

  /**
   * Lookup a previously decoded transaction from the backend index.
   */
  async lookupIndexedTx(
    chain: string,
    hash: string,
  ): Promise<NormalizedTransaction> {
    const url = `${this.baseUrl}/api/index/${chain}/${hash}`;
    const res = await this.fetchImpl(url, {
      headers: this.getHeaders(),
    });

    if (!res.ok) {
      throw await this.handleError(res);
    }

    const body = (await res.json()) as DecodeSuccessEnvelope;
    return body.decoded;
  }

  /**
   * List recent transactions decoded and indexed by the API.
   */
  async listRecentIndexedTxs(limit: number = 20): Promise<NormalizedTransaction[]> {
    const url = `${this.baseUrl}/api/index/recent?limit=${limit}`;
    const res = await this.fetchImpl(url, {
      headers: this.getHeaders(),
    });

    if (!res.ok) {
      throw await this.handleError(res);
    }

    const body = (await res.json()) as IndexedListResponse;
    return body.items;
  }

  private async get<T>(path: string): Promise<T> {
    const url = `${this.baseUrl}${path}`;
    const res = await this.fetchImpl(url, {
      headers: this.getHeaders(),
    });

    if (!res.ok) {
      throw await this.handleError(res);
    }

    return (await res.json()) as T;
  }

  private getHeaders(): Record<string, string> {
    return {
      ...(this.apiKey ? { "x-api-key": this.apiKey } : null),
    };
  }

  private buildSearchParams(params: { chain: string; hash: string; rpcUrl?: string }): URLSearchParams {
    const search = new URLSearchParams({
      chain: params.chain,
      hash: params.hash.trim(),
    });

    if (params.rpcUrl?.trim()) {
      search.set("rpc_url", params.rpcUrl.trim());
    }
    return search;
  }

  private async handleError(res: Response): Promise<Error> {
    const body = (await res.json()) as DecodeErrorEnvelope;
    const error = new Error(body.error.message);
    (error as any).code = body.error.code;
    (error as any).retryable = body.error.retryable;
    return error;
  }
}
