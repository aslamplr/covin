import React from "react";
import { getDistricts, findCenters, District, Center, VaccineType, VaccineTypes } from "../services/api";
import DistrictSelect from "../components/DistrictSelect";
import Centers from "../components/Centers";
import VaccSelect from "../components/VaccSelect";

export default function App() {
  const [districts, setDistricts] = React.useState<District[]>([]);
  const [centers, setCenters] = React.useState<Center[]>([]);
  const [vaccType, setVaccType] = React.useState<VaccineType>(VaccineTypes[0]);
  const [district, setDistrict] = React.useState<District | undefined>(
    undefined
  );

  React.useEffect(() => {
    getDistricts().then((districts) => {
      setDistricts(districts);
    });
  }, []);

  const callGetCenters = (dist?: District, vacc: VaccineType = VaccineTypes[0]) => {
    setVaccType(vacc);
    if (dist) {
      setDistrict(dist);
      findCenters(dist.district_id, vacc).then(({ centers }) => {
        setCenters(centers);
      });
    }
  };

  const onDistrictSelected = (district: District) => {
    callGetCenters(district, vaccType);
  };

  const onVaccSelected = (vacc: VaccineType) => {
    callGetCenters(district, vacc);
  };

  return (
    <div>
      <header className="bg-white shadow">
        <div className="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
          <h1 className="text-3xl font-bold text-gray-900">Locator</h1>
        </div>
      </header>
      <main>
        <div className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
          {districts && districts.length > 0 && (
            <>
              <DistrictSelect
                selected={district}
                districts={districts}
                onSelected={onDistrictSelected}
              />
              <VaccSelect
                selected={vaccType}
                onSelected={onVaccSelected}
                vaccTypes={VaccineTypes}
              />
            </>
          )}
          {centers && centers.length > 0 && <Centers centers={centers} />}
          {centers && centers.length === 0 && (
            <div className="text-lg text-gray-400">No centers found</div>
          )}
        </div>
      </main>
    </div>
  );
}
