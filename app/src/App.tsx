import React from "react";
import { getDistricts, getCenters, District, Center } from "./services/api";
import DistrictSelect from "./components/DistrictSelect";
import Centers from "./components/Centers";

export default function App() {
  const [districts, setDistricts] = React.useState<District[]>([]);
  const [centers, setCenters] = React.useState<Center[]>([]);

  React.useEffect(() => {
    getDistricts().then((districts) => {
      setDistricts(districts);
    });
  }, []);

  const onDistrictSelected = (district: District) => {
    getCenters(district.district_id).then(({ centers }) => {
      setCenters(centers);
    });
  };

  return (
    <div>
      <header className="bg-white shadow">
        <div className="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
          <h1 className="text-3xl font-bold text-gray-900"> 📍 Covin Locator</h1>
        </div>
      </header>
      <main>
        <div className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
          {districts && districts.length > 0 && (
            <DistrictSelect
              districts={districts}
              onSelected={onDistrictSelected}
            />
          )}
          {centers && centers.length > 0 && <Centers centers={centers} />}
          {centers && centers.length === 0 && <div className="text-lg text-gray-400">No centers found</div>}
        </div>
      </main>
    </div>
  );
}
