import { getAccessJwtToken } from "./auth";

const SETU_BASE_URL = process.env.REACT_APP_SETU_BASE_URL;
const BASE_URL = process.env.REACT_APP_API_BASE_URL;
const ALL_CENTERS_URL = process.env.REACT_APP_ALL_CENTERS_URL;
const ALL_DISTRICTS_URL = process.env.REACT_APP_ALL_DISTRICTS_URL;

async function publicFetch(
  input: RequestInfo,
  init?: RequestInit
): Promise<Response> {
  const resp = await fetch(input, init);
  if (resp.status >= 200 && resp.status < 300) {
    return resp;
  } else {
    throw resp;
  }
}

async function authFetch(
  input: RequestInfo,
  init?: RequestInit
): Promise<Response> {
  const bearerToken = await getAccessJwtToken();
  const authHeaders = {
    Authorization: `Bearer ${bearerToken}`,
  };
  const reqInit: RequestInit = init
    ? {
        ...init,
        headers: {
          ...init.headers,
          ...authHeaders,
        },
      }
    : {
        headers: authHeaders,
      };
  return publicFetch(input, reqInit);
}

export enum VaccineType {
  ANY = "ANY",
  COVISHIELD = "COVISHIELD",
  COVAXIN = "COVAXIN",
}

export const VaccineTypes = [
  VaccineType.ANY,
  VaccineType.COVISHIELD,
  VaccineType.COVAXIN,
];
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
  min_age_limit: number;
  date: string;
  slots: string[];
}

export interface CenterDim {
  centerId: number;
  name: string;
  districtId: number;
  stateId: number;
}

function padString(numStr: number, padStr: string, len: number): string {
  let str = numStr.toString();
  while (str.length < len) str = padStr + str;
  return str;
}

let allCenters: Array<CenterDim> | undefined;

export async function getAllCenters(): Promise<CenterDim[]> {
  if (allCenters && allCenters.length) {
    console.info(`Serving allCenters from api::cache!`);
    return allCenters;
  }
  try {
    const resp = await publicFetch(ALL_CENTERS_URL!);
    const json = await resp.json();
    allCenters = json;
    return json;
  } catch (error) {
    console.error("An error occured", error);
    throw Error("An error occured");
  }
}

export async function getCenters(districtId: number): Promise<CenterDim[]> {
  const centers = await getAllCenters();
  return centers.filter(({ districtId: distId }) => districtId === distId);
}

export async function findCenters(
  districtId: number = 296,
  vaccine: VaccineType
): Promise<CenterResponse> {
  const vaccTypeQuery =
    vaccine === VaccineType.ANY ? "" : `&vaccine=${vaccine}`;
  const currentDate = new Date();
  currentDate.setDate(currentDate.getDate() + 1);
  const date = `${padString(currentDate.getDate(), "0", 2)}-${padString(
    currentDate.getMonth() + 1,
    "0",
    2
  )}-${currentDate.getFullYear()}`;
  try {
    try {
      const resp = await publicFetch(
        `${SETU_BASE_URL}/v2/appointment/sessions/calendarByDistrict?district_id=${districtId}&date=${date}${vaccTypeQuery}`
      );
      const json = await resp.json();
      return json;
    } catch (error) {
      console.warn(error);
      const resp = await publicFetch(
        `${BASE_URL}/centers?district_id=${districtId}&date=${date}${vaccTypeQuery}`
      );
      const json = await resp.json();
      return json;
    }
  } catch (error) {
    console.error("An error occured,", error);
    throw Error("An error occured");
  }
}

let allDistricts: Array<District> | undefined;

export async function getDistricts(): Promise<District[]> {
  if (allDistricts && allDistricts.length) {
    return allDistricts.filter(({ state_id }) => state_id === 17);
  }
  try {
    try {
      const resp = await publicFetch(ALL_DISTRICTS_URL!);
      const json: District[] = await resp.json();
      allDistricts = json;
      return json.filter(({ state_id }) => state_id === 17);
    } catch (error) {
      console.warn(error);
      const resp = await publicFetch(`${BASE_URL}/districts`);
      const json: District[] = await resp.json();
      allDistricts = json;
      return json.filter(({ state_id }) => state_id === 17);
    }
  } catch (error) {
    console.error("An error occured,", error);
    throw Error("An error occured");
  }
}

export interface Alert {
  centers: number[];
  districtId: number;
  email: string;
  mobileNo: string;
  age: number;
}

export async function getAlert(): Promise<Alert|undefined> {
  try {
    const resp = await authFetch(`${BASE_URL}/alerts/register`);
    const json: Alert = await resp.json();
    return json;
  } catch (error) {
    if (error.status && error.status === 404) {
      return;
    }
    console.error("An error occured", error);
    throw Error("An error occured");
  }
}

export async function createAlert(alert: Alert): Promise<void> {
  try {
    await authFetch(`${BASE_URL}/alerts/register`, {
      method: "POST",
      mode: "cors",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(alert),
    });
  } catch (error) {
    console.error("An error occured", error);
    throw Error("An error occured");
  }
}

export async function deleteAlert(): Promise<void> {
  try {
    await authFetch(`${BASE_URL}/alerts/register`, {
      method: "DELETE",
      mode: "cors",
    });
  } catch (error) {
    console.error("An error occured", error);
    throw Error("An error occured");
  }
}
