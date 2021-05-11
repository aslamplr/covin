import React from "react";
import { Alert, CenterDim, District } from "../../services/api";
import { useFormik } from "formik";
import * as yup from "yup";

interface Props {
  alert?: Alert;
  districts: District[];
  centers?: CenterDim[];
  cancelEdit: () => void;
  onSubmit: (alert: Alert) => void;
  onDistrictSelect: (districtId: number) => void;
}

export default function AlertsEdit({
  alert,
  districts,
  centers,
  cancelEdit,
  onSubmit,
  onDistrictSelect,
}: Props) {
  const formik = useFormik({
    initialValues: alert
      ? { ...alert, mobileNo: alert.mobileNo.substr(3, 10) }
      : {
          districtId: 0,
          centers: [],
          email: "",
          mobileNo: "",
          age: "",
        },
    validationSchema: yup.object({
      districtId: yup.number().required(),
      centers: yup.array().of(yup.number()).min(1).max(20).required(),
      email: yup.string().email(),
      mobileNo: yup.string().matches(/^[6-9]\d{9}$/),
      age: yup.number().min(18).max(150),
    }),
    onSubmit: (values) => {
      onSubmit({
        districtId: Number(values.districtId),
        centers: values.centers.map(Number),
        email: values.email,
        mobileNo: `+91${values.mobileNo}`,
        age: Number(values.age),
      });
    },
  });

  const centerMap: { [key: number]: string } = (centers || []).reduce(
    (previousVal, { centerId, name }) => {
      return {
        ...previousVal,
        [centerId]: name,
      };
    },
    {}
  );

  const onDistrictValueChange = (
    event: React.ChangeEvent<HTMLSelectElement>
  ) => {
    onDistrictSelect(parseInt(event.target.value));
    formik.handleChange(event);
  };

  return (
    <div className="antialiased font-sans bg-gray-200">
      <div className="bg-gray-100">
        <div className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
          <div className="mt-5 md:mt-0 md:col-span-2">
            <form onSubmit={formik.handleSubmit}>
              <div className="shadow sm:rounded-md sm:overflow-hidden">
                <div className="px-4 py-5 bg-white space-y-6 sm:p-6">
                  <div className="col-span-6 sm:col-span-3">
                    <label
                      htmlFor="districtId"
                      className="block text-sm font-medium text-gray-700"
                    >
                      District
                    </label>
                    <select
                      id="districtId"
                      name="districtId"
                      onChange={onDistrictValueChange}
                      value={formik.values.districtId}
                      className="mt-1 block w-full py-2 px-3 border border-gray-300 bg-white rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                    >
                      {!alert && <option value={0}>Select District</option>}
                      {districts.map(({ district_id, district_name }) => (
                        <option key={district_id} value={district_id}>
                          {district_name}
                        </option>
                      ))}
                    </select>
                    {formik.touched.districtId && formik.errors.districtId ? (
                      <div className="text-red-700 text-sm m-2 p-2">
                        {formik.errors.districtId}
                      </div>
                    ) : null}
                  </div>

                  <div className="col-span-6 sm:col-span-3">
                    <label
                      htmlFor="centers"
                      className="block text-sm font-medium text-gray-700"
                    >
                      Centers
                    </label>
                    <p className="m-2 text-gray-500 text-sm">
                      Use `Ctrl` or `⌘` key to select multiple centers (upto 20
                      centers).
                    </p>
                    <p className="m-2">
                      {formik.values.centers.map((centerId: number) => (
                        <span
                          key={centerId}
                          className="text-gray-100 m-1 p-1 bg-purple-400 text-sm border rounded-md border-gray-400"
                        >
                          {centerMap[centerId]}
                        </span>
                      ))}
                    </p>
                    <div className="mt-1 flex rounded-md shadow-sm">
                      <select
                        id="centers"
                        name="centers"
                        onChange={formik.handleChange}
                        value={formik.values.centers as unknown as string[]}
                        className="form-multiselect shadow-sm focus:ring-indigo-500 focus:border-indigo-500 mt-1 block w-full sm:text-sm border-gray-300 rounded-md"
                        multiple
                      >
                        {centers &&
                          centers.map(({ centerId, name }) => (
                            <option key={centerId} value={centerId}>
                              {name}
                            </option>
                          ))}
                      </select>
                    </div>
                    {formik.touched.centers && formik.errors.centers ? (
                      <div className="text-red-700 text-sm m-2 p-2">
                        {formik.errors.centers}
                      </div>
                    ) : null}
                  </div>

                  <div className="col-span-6 sm:col-span-4">
                    <label
                      htmlFor="email"
                      className="block text-sm font-medium text-gray-700"
                    >
                      Email address
                    </label>
                    <input
                      type="text"
                      name="email"
                      id="email"
                      autoComplete="email"
                      onChange={formik.handleChange}
                      value={formik.values.email}
                      className="mt-1 focus:ring-indigo-500 focus:border-indigo-500 block w-full shadow-sm sm:text-sm border-gray-300 rounded-md"
                    />
                    {formik.touched.email && formik.errors.email ? (
                      <div className="text-red-700 text-sm m-2 p-2">
                        {formik.errors.email}
                      </div>
                    ) : null}
                  </div>

                  <div className="col-span-6 sm:col-span-3">
                    <label
                      htmlFor="mobile"
                      className="block text-sm font-medium text-gray-700"
                    >
                      Mobile Number
                    </label>
                    <div className="mt-1 flex rounded-md shadow-sm">
                      <span className="inline-flex items-center px-3 rounded-l-md border border-r-0 border-gray-300 bg-gray-50 text-gray-500 text-sm">
                        + 91
                      </span>
                      <input
                        type="text"
                        name="mobileNo"
                        id="mobileNo"
                        onChange={formik.handleChange}
                        value={formik.values.mobileNo}
                        className="focus:ring-indigo-500 focus:border-indigo-500 flex-1 block w-full rounded-none rounded-r-md sm:text-sm border-gray-300"
                        placeholder="10 digit mobile number"
                      />
                    </div>
                    {formik.touched.mobileNo && formik.errors.mobileNo ? (
                      <div className="text-red-700 text-sm m-2 p-2">
                        {formik.errors.mobileNo}
                      </div>
                    ) : null}
                  </div>

                  <div className="col-span-3 sm:col-span-2">
                    <label
                      htmlFor="age"
                      className="block text-sm font-medium text-gray-700"
                    >
                      Age
                    </label>
                    <input
                      type="text"
                      name="age"
                      id="age"
                      onChange={formik.handleChange}
                      value={formik.values.age}
                      className="mt-1 focus:ring-indigo-500 focus:border-indigo-500 block w-full shadow-sm sm:text-sm border-gray-300 rounded-md"
                    />
                    {formik.touched.age && formik.errors.age ? (
                      <div className="text-red-700 text-sm m-2 p-2">
                        {formik.errors.age}
                      </div>
                    ) : null}
                  </div>
                </div>

                {/* <div className="px-4 py-5 bg-white space-y-6 sm:p-6">
                  <fieldset>
                    <div>
                      <legend className="text-base font-medium text-gray-900">
                        Deliver through
                      </legend>
                      <p className="text-sm text-gray-500">
                        These are delivered via Email or SMS (to your mobile
                        number, when possible).
                      </p>
                    </div>
                    <div className="mt-4 space-y-4">
                      <div className="flex items-center">
                        <input
                          id="push_everything"
                          name="push_notifications"
                          type="radio"
                          className="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300"
                        />
                        <div className="ml-3 text-sm">
                          <label
                            htmlFor="offers"
                            className="font-medium text-gray-700"
                          >
                            Always Email
                          </label>
                          <p className="text-gray-500">
                            Get notified only via email.
                          </p>
                        </div>
                      </div>
                      <div className="flex items-center">
                        <input
                          id="push_email"
                          name="push_notifications"
                          type="radio"
                          className="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300"
                        />
                        <div className="ml-3 text-sm">
                          <label
                            htmlFor="offers"
                            className="font-medium text-gray-700"
                          >
                            Always SMS
                          </label>
                          <p className="text-gray-500">
                            Get notified only via SMS. These might not arrive as
                            we are working on SMS notification, once ready will
                            deliver through SMS
                          </p>
                        </div>
                      </div>
                      <div className="flex items-center">
                        <input
                          id="push_nothing"
                          name="push_notifications"
                          type="radio"
                          className="focus:ring-indigo-500 h-4 w-4 text-indigo-600 border-gray-300"
                        />
                        <div className="ml-3 text-sm">
                          <label
                            htmlFor="offers"
                            className="font-medium text-gray-700"
                          >
                            SMS when possible and fallback to Email
                          </label>
                          <p className="text-gray-500">
                            Get notified via SMS when possible, and fallback to
                            delivering via Email when SMS delivery is not
                            available.
                          </p>
                        </div>
                      </div>
                    </div>
                  </fieldset>
                </div> */}

                <div className="px-4 py-3 bg-gray-50 text-right sm:px-6">
                  <button
                    onClick={cancelEdit}
                    className="mr-2 inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-gray-900 bg-white hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    className="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                  >
                    Save
                  </button>
                </div>
              </div>
            </form>
          </div>
        </div>
      </div>
    </div>
  );
}
