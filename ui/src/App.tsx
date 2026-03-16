import { Routes, Route, Navigate } from "react-router-dom";
import { useConnection } from "./stores/connection";
import ConnectScreen from "./components/connect/ConnectScreen";
import Layout from "./components/layout/Layout";

export default function App() {
  const status = useConnection((s) => s.status);

  return (
    <Routes>
      <Route path="/" element={<ConnectScreen />} />
      <Route
        path="/app/*"
        element={status === "connected" ? <Layout /> : <Navigate to="/" />}
      />
    </Routes>
  );
}
