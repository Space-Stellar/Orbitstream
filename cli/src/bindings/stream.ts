export class Client {
  constructor(config: any) {}
  async init(args: any): Promise<any> { return {}; }
  async get_stream(args: any): Promise<any> { 
    return { token: "MOCK", flow_rate: BigInt(100) }; 
  }
  async claim(args: any): Promise<any> { return {}; }
}
