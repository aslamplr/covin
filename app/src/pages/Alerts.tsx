import React from "react";
import { getAccessJwtToken } from "../services/auth";
import { getAlert, Alert } from "../services/api";

export default function Alerts() {
  const [jwtToken, setJwtToken] = React.useState<string | undefined>();
  const [alert, setAlert] = React.useState<Alert | undefined>();
  React.useEffect(() => {
    getAccessJwtToken().then((token: string) => {
      setJwtToken(token);
    });
    getAlert().then((alert) => {
      setAlert(alert);
    });
  }, []);
  return (
    <div className="">
      <pre>{jwtToken}</pre>
      <pre>{JSON.stringify(alert, null, 2)}</pre>
    </div>
  );
}
