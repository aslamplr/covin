import { Center } from "../services/api";

interface Props {
  centers: Center[];
}

export default function Centers({ centers }: Props) {
  return (
    <div className="flex flex-col">
      <div className="-my-2 overflow-x-auto sm:-mx-6 lg:-mx-8">
        <div className="py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8">
          <div className="shadow overflow-hidden border-b border-gray-200 sm:rounded-lg">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th
                    scope="col"
                    className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                  >
                    Name
                  </th>
                  <th
                    scope="col"
                    className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                  >
                    Time (from - to)
                  </th>
                  <th
                    scope="col"
                    className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                  >
                    Availability
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {centers.map((center) => (
                  <tr key={center.center_id}>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="text-sm font-medium text-gray-900">
                        {center.name}
                      </div>
                      <div className="text-sm text-gray-500">
                        Block: {center.block_name}
                      </div>
                      <div className="text-sm text-gray-500">
                        Pin: {center.pincode}
                      </div>
                      <div className="text-sm text-gray-500">
                        📍 Lat: {center.lat}; Long: {center.long}
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="text-sm text-gray-800">{center.from}</div>
                      <div className="text-sm text-gray-800">{center.to}</div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      {center.sessions.map((session) => (
                        <div
                          key={session.session_id}
                          className="mb-2 bg-white shadow overflow-hidden sm:rounded-lg"
                        >
                          <div className="px-4 py-5 sm:px-6">
                            <h3 className="text-lg leading-6 font-medium text-gray-900">
                              {session.date}
                            </h3>
                            <div className="text-sm text-gray-800">
                              Available Capacity: {session.available_capacity}
                            </div>
                            <div className="text-sm text-gray-800">
                              Slots: {session.slots.join(", ")}
                            </div>
                          </div>
                        </div>
                      ))}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  );
}
