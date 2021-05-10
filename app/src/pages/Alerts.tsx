import React from "react";
import { getAlert, Alert, CenterDim, getCenters } from "../services/api";
import AlertEdit from "../components/alerts/AlertEdit";
import AlertView from "../components/alerts/AlertView";

export default function Alerts() {
  const [alert, setAlert] = React.useState<Alert | undefined>();
  const [ centers, setCenters ] = React.useState<CenterDim[] | undefined>();
  const [isEdit, setEditing] = React.useState<boolean>(false);
  React.useEffect(() => {
    getAlert().then((alert) => {
      setAlert(alert);
    });
    getCenters(296).then((centers) => {
      setCenters(centers);
    });
  }, []);
  return (
    <div>
      <div>
        <div className="m-7 bg-white shadow overflow-hidden sm:rounded-lg">
          <pre className="m-2 p-2 text-blue-500">
            {JSON.stringify(centers, null, 2)}
          </pre>
        </div>
      </div>
      <div>
        <div className="m-7 bg-white shadow overflow-hidden sm:rounded-lg">
          <pre className="m-2 p-2 text-blue-500">
            {JSON.stringify(alert, null, 2)}
          </pre>
        </div>
      </div>
      <div className="antialiased font-sans bg-gray-200">
        <div className="bg-gray-100">
          <div className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
            {alert && !isEdit ? (
              <AlertView setEditing={() => setEditing(true)} alert={alert} />
            ) : (
              <AlertEdit alert={alert}/>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
