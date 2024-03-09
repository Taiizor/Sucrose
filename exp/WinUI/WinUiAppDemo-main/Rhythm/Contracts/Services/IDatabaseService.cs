using Oracle.ManagedDataAccess.Client;

namespace Rhythm.Contracts.Services;

internal interface IDatabaseService
{
    OracleConnection? Connection
    {
        get;
    }

    void ConnectToOracle();

    void DisconnectFromOracle();

    OracleConnection GetOracleConnection();
}
