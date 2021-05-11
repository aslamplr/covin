import React from "react";
import { Alert, CenterDim, District } from "../../services/api";

interface Props {
  alert: Alert;
  districts: District[];
  centers: CenterDim[];
  setEditing: () => void;
}

export default function AlertView({
  alert,
  districts,
  centers,
  setEditing,
}: Props) {
  const districtMap: { [key: number]: string } = districts.reduce(
    (previousVal, { district_id, district_name }) => {
      return {
        ...previousVal,
        [district_id]: district_name,
      };
    },
    {}
  );

  const centerMap: { [key: number]: string } = (centers || []).reduce(
    (previousVal, { centerId, name }) => {
      return {
        ...previousVal,
        [centerId]: name,
      };
    },
    {}
  );

  return (
    <div className="bg-white shadow overflow-hidden sm:rounded-lg">
      <div className="px-4 py-5 sm:px-6 relative">
        <h3 className="text-lg leading-6 font-medium text-gray-900">
          Alert Settings
        </h3>
        <p className="mt-1 max-w-2xl text-sm text-gray-500">
          Currently alert configuration.
        </p>
        <button
          onClick={setEditing}
          type="button"
          className="absolute top-7 right-3 inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
        >
          Edit
        </button>
      </div>
      <div className="border-t border-gray-200">
        <dl>
          <div className="bg-gray-50 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
            <dt className="text-sm font-medium text-gray-500">District</dt>
            <dd className="mt-1 text-sm text-gray-900 sm:mt-0 sm:col-span-2">
              {districtMap[alert.districtId]}
            </dd>
          </div>
          <div className="bg-white px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
            <dt className="text-sm font-medium text-gray-500">
              Selected Centers
            </dt>
            <dd className="mt-1 text-sm text-gray-900 sm:mt-0 sm:col-span-2">
              {alert.centers.map((centerId) => centerMap[centerId]).join("; ")}
            </dd>
          </div>
          <div className="bg-gray-50 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
            <dt className="text-sm font-medium text-gray-500">Email</dt>
            <dd className="mt-1 text-sm text-gray-900 sm:mt-0 sm:col-span-2">
              {alert.email}
            </dd>
          </div>
          <div className="bg-white px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
            <dt className="text-sm font-medium text-gray-500">Mobile</dt>
            <dd className="mt-1 text-sm text-gray-900 sm:mt-0 sm:col-span-2">
              {alert.mobileNo.substr(3, 10)}
            </dd>
          </div>
          <div className="bg-gray-50 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
            <dt className="text-sm font-medium text-gray-500">Age</dt>
            <dd className="mt-1 text-sm text-gray-900 sm:mt-0 sm:col-span-2">
              {alert.age} years.
            </dd>
          </div>
        </dl>
      </div>
    </div>
  );
}
