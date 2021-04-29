const BASE_URL = process.env.REACT_APP_API_BASE_URL;

export interface District {
  district_name: string;
  district_id: number;
  state_id: number;
}

export interface CenterResponse {
  centers: Center[];
}

export interface Center {
  center_id: string;
  name: string;
  state_name: string;
  district_name: string;
  block_name: string;
  pincode: number;
  from: string;
  to: string;
  lat: number;
  long: number;
  fee_type: string;
  sessions: Session[];
}

export interface Session {
  session_id: string;
  available_capacity: number;
  date: string;
  slots: string[];
}

function padString(numStr: number, padStr: string, len: number): string {
  let str = numStr.toString();
  while (str.length < len) str = padStr + str;
  return str;
}

export async function getCenters(
  districtId: number = 296,
  vaccine: string = "COVISHIELD"
): Promise<CenterResponse> {
  const currentDate = new Date();
  currentDate.setDate(currentDate.getDate() + 1);
  const date = `${padString(currentDate.getDate(), "0", 2)}-${padString(
    currentDate.getMonth() + 1,
    "0",
    2
  )}-${currentDate.getFullYear()}`;
  try {
    const resp = await fetch(
      `${BASE_URL}/centers?district_id=${districtId}&date=${date}&vaccine=${vaccine}`
    );
    const json = await resp.json();
    return json;
  } catch (error) {
    console.error("An error occured,", error);
    throw Error("An error occured");
  }
}

export async function getDistricts(): Promise<District[]> {
  try {
    const resp = await fetch(`${BASE_URL}/districts`);
    const json: District[] = await resp.json();
    return json.filter(({ state_id, district_name }) => state_id === 17);
  } catch (error) {
    console.error("An error occured,", error);
    throw Error("An error occured");
  }
}
