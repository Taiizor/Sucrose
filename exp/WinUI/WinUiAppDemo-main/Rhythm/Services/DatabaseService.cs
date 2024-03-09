using Oracle.ManagedDataAccess.Client;
using Rhythm.Contracts.Services;

namespace Rhythm.Services;

public class DatabaseNullException : Exception
{
    public DatabaseNullException(string message) : base(message)
    {

    }
}

public class DatabaseService : IDatabaseService
{

    private readonly string connectionString = "ORACLE_CONNECTION_STRING";

    public OracleConnection? Connection
    {
        get;
        private set;
    }

    public DatabaseService()
    {
        ConnectToOracle();
    }

    public void ConnectToOracle()
    {
        if (Connection != null)
        {
            Connection = new OracleConnection(connectionString);
            Connection.Open();
        }
    }

    public void DisconnectFromOracle()
    {
        if (Connection != null)
        {
            Connection.Dispose();
        }
    }

    public OracleConnection GetOracleConnection()
    {
        if (Connection == null)
        {
            throw new DatabaseNullException("Oracle Connection is null");
        }
        return Connection;
    }
}
