import React from "react";
import { getAccessJwtToken } from "../services/auth";

export default function Alerts() {
  const [jwtToken, setJwtToken] = React.useState<string | undefined>();
  React.useEffect(() => {
    getAccessJwtToken().then((token: string) => {
      setJwtToken(token);
    });
  }, []);
  return (
    <div className="">
      <pre>{jwtToken}</pre>
    </div>
  );
}
