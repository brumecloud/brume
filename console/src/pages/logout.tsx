import { useEffect } from "react";
import { useNavigate } from "react-router-dom";

export const Logout = () => {
	const navigate = useNavigate();

	useEffect(() => {
		navigate("/login");
	}, []);

	return null;
};
