import Cookies from "js-cookie";

export const setToken = (token: string) => {
  Cookies.set("access_token", token);
};

export const getToken = () => {
  const token = Cookies.get("access_token");
  return token;
};

export const setRefreshToken = (token: string) => {
  Cookies.set("refresh_token", token);
};

export const getRefreshToken = () => {
  const token = Cookies.get("refresh_token");
  return token;
};
