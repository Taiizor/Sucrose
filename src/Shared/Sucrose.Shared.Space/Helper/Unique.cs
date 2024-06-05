using System.Security.Cryptography;
using SEET = Skylark.Enum.EncodeType;
using SHE = Skylark.Helper.Encode;
using SHG = Skylark.Helper.Guidly;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Unique
    {
        public static Guid Generate(string Value)
        {
            return SHG.ByteToGuid(MD5.Create().ComputeHash(SHE.GetBytes(Value, SEET.UTF8)));
        }

        public static string GenerateText(string Value)
        {
            return SHG.GuidToText(SHG.ByteToGuid(MD5.Create().ComputeHash(SHE.GetBytes(Value, SEET.UTF8))));
        }
    }
}