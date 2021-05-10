import React from "react";
import { getAccessJwtToken } from "../../services/auth";

export default function TokenView() {
  const [jwtToken, setJwtToken] = React.useState<string | undefined>();
  React.useEffect(() => {
    getAccessJwtToken().then((token: string) => {
      setJwtToken(token);
    });
  }, []);
  return (
    <div className="bg-gray-100">
      <div className="m-7 p-4 bg-white shadow overflow-hidden sm:rounded-lg">
        <div className="h-4 text-gray-700">token:</div>
        <pre className="m-2 p-2 text-pink-900 overflow-scroll break-words">
          {jwtToken}
        </pre>
      </div>
    </div>
  );
}
