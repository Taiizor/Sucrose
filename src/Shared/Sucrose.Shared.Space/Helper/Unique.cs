using System.Security.Cryptography;
using SEET = Skylark.Enum.EncodeType;
using SHE = Skylark.Helper.Encode;
using SHG = Skylark.Helper.Guidly;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Unique
    {
        public static Guid GenerateGuid(string Value)
        {
            return SHG.ByteToGuid(MD5.Create().ComputeHash(SHE.GetBytes(Value, SEET.UTF8)));
        }
    }
}