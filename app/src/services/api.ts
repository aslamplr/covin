import { getAccessJwtToken } from "./auth";

const BASE_URL = process.env.REACT_APP_API_BASE_URL;

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

function padString(numStr: number, padStr: string, len: number): string {
  let str = numStr.toString();
  while (str.length < len) str = padStr + str;
  return str;
}

export async function getCenters(
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
    const resp = await publicFetch(
      `${BASE_URL}/centers?district_id=${districtId}&date=${date}${vaccTypeQuery}`
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
    const resp = await publicFetch(`${BASE_URL}/districts`);
    const json: District[] = await resp.json();
    return json.filter(({ state_id }) => state_id === 17);
  } catch (error) {
    console.error("An error occured,", error);
    throw Error("An error occured");
  }
}

export interface Alert {
  location: {
    lat: number;
    long: number;
  };
  districtId: number;
  email: string;
  mobileNo: string;
  age: number;
  kilometers: number;
}

export async function getAlert(): Promise<Alert> {
  try {
    const resp = await authFetch(`${BASE_URL}/alerts/register`);
    const json: Alert = await resp.json();
    return json;
  } catch (error) {
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
