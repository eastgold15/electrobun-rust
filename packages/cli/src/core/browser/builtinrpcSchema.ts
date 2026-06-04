// consider just makeing a shared types file

export interface BuiltinBunToWebviewSchema {
  requests: {
    evaluateJavascriptWithResponse: {
      params: { script: string };
      response: any;
    };
  };
}

export interface BuiltinWebviewToBunSchema {
  requests: {
    webviewTagInit: {
      params: {};
      response: any;
    };
  };
}
