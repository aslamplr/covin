import React from "react";
import {
  getAlert,
  Alert,
  CenterDim,
  getCenters,
  getAllCenters,
  getDistricts,
  createAlert as postAlert,
  District,
  deleteAlert as removeAlert,
} from "../services/api";
import AlertEdit from "../components/alerts/AlertEdit";
import AlertView from "../components/alerts/AlertView";

export default function Alerts() {
  const [loading, setLoading] = React.useState<boolean>(true);
  const [alert, setAlert] = React.useState<Alert | undefined>();
  const [districts, setDistricts] = React.useState<District[] | undefined>();
  const [centers, setCenters] = React.useState<CenterDim[] | undefined>();
  const [isEdit, setEditing] = React.useState<boolean>(false);

  const initialize = () => {
    Promise.all([getAlert(), getDistricts(), getAllCenters()]).then(
      ([alert, districts]) => {
        if (alert) {
          getCenters(alert.districtId).then((centers) => {
            setCenters(centers);
          });
        }
        setAlert(alert);
        setDistricts(districts);
        setLoading(false);
      }
    );
  };

  React.useEffect(() => {
    initialize();
  }, []);

  const onDistrictSelect = (districtId: number) => {
    getCenters(districtId).then((centers) => {
      setCenters(centers);
    });
  };

  const createAlert = (alert: Alert) => {
    postAlert(alert).then(() => {
      initialize();
      setEditing(false);
    });
  };

  const deleteAlert = () => {
    if (
      window.confirm(
        "Are you sure you want to delete the alert?\nPress 'OK' to delete the alert, 'Cancel' otherwise!"
      )
    ) {
      removeAlert().then(() => {
        initialize();
        setEditing(false);
      });
    }
  };

  return (
    <div>
      {/* <div>
        <div className="m-7 bg-white shadow overflow-hidden sm:rounded-lg">
          <pre className="m-2 p-2 text-blue-500">
            {JSON.stringify(alert, null, 2)}
          </pre>
        </div>
      </div> */}
      <div className="antialiased font-sans bg-gray-200">
        <div className="bg-gray-100">
          <div className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
            {loading ? (
              "Loading..."
            ) : alert && !isEdit ? (
              <AlertView
                deleteAlert={() => deleteAlert()}
                setEditing={() => setEditing(true)}
                alert={alert}
                districts={districts!}
                centers={centers!}
              />
            ) : (
              <AlertEdit
                onDistrictSelect={onDistrictSelect}
                onSubmit={createAlert}
                cancelEdit={() => setEditing(false)}
                alert={alert}
                districts={districts!}
                centers={centers!}
              />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
