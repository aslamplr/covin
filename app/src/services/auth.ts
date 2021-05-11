import Amplify, { Auth } from "aws-amplify";

export const configure = () => {
  Amplify.configure({
    Auth: {
      region: "ap-south-1",
      userPoolId: "ap-south-1_0DvxhDRsV",
      userPoolWebClientId: "68uau6menju7q3prl3t3gr1ksu",
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
