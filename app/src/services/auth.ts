import Amplify, { Auth } from "aws-amplify";

const COGNITO_REGION = process.env.REACT_APP_COGNITO_REGION;
const COGNITO_POOL_ID = process.env.REACT_APP_COGNITO_POOL_ID;
const COGNITO_WEB_CLIENT_ID = process.env.REACT_APP_COGNITO_WEB_CLIENT_ID;

export const configure = () => {
  Amplify.configure({
    Auth: {
      region: COGNITO_REGION,
      userPoolId: COGNITO_POOL_ID,
      userPoolWebClientId: COGNITO_WEB_CLIENT_ID,
    },
  });
};

export const getCurrentSession = async () => {
  return Auth.currentSession();
};

export const getAccessJwtToken = async () => {
  const currSession = await getCurrentSession();
  const accessToken = currSession.getAccessToken();
  const jwt = accessToken.getJwtToken();
  return jwt;
};
