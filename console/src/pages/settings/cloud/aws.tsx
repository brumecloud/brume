import { useLazyQuery, useMutation } from "@apollo/client";
import { useState } from "react";
import { FaArrowRightLong } from "react-icons/fa6";
import { MdChecklist, MdOutlineLocalPolice, MdPreview } from "react-icons/md";
import { TfiPackage } from "react-icons/tfi";
import { useNavigate } from "react-router-dom";
import { CloudProvider } from "@/_apollo/graphql";
import { Page } from "@/components/page-comp/header";
import { Stepper } from "@/components/stepper";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  CREATE_CLOUD_ACCOUNT,
  GET_AWS_CLOUD_FORMATION_URL,
} from "@/gql/cloud.graphql";

export const AwsPage = () => {
  const navigate = useNavigate();
  const [awsAccount, setAwsAccount] = useState<string>("913355241065");
  const [awsName, setAwsName] = useState<string>("TestCloud - AWS");
  const [agreement, setAgreement] = useState<boolean>(false);

  const [getUrl] = useLazyQuery(GET_AWS_CLOUD_FORMATION_URL);
  const [startEndToEndTestMutation, { loading: accountCreationLoading }] =
    useMutation(CREATE_CLOUD_ACCOUNT);

  const beginEnrollingAccount = async () => {
    const { data, errors } = await startEndToEndTestMutation({
      variables: {
        name: awsName,
        cloudAccountId: awsAccount,
        cloudProvider: CloudProvider.Aws,
      },
    });

    if (errors || !data) {
      console.error(errors);
      return;
    }

    const id = data.createCloudAccount.id;

    setTimeout(() => {
      navigate(`/settings/cloud/${id}`);
    }, 500);
  };

  const redirectToAWS = async () => {
    const { data, error } = await getUrl({
      fetchPolicy: "network-only",
    });

    if (error || !data) {
      console.error(error);
      return;
    }
    window.open(data.getAWSCloudFormationURL, "_blank");
  };

  return (
    <Page.Container>
      <Page.Header>
        <Page.Title>Connect your Amazon Web Service account</Page.Title>
        <Page.Description>
          Lets authorize Brume Cloud to operate on your AWS account
        </Page.Description>
      </Page.Header>
      <Page.Body className="h-full pt-16">
        <Stepper.Root leftBorder>
          <Stepper.Item>
            <Stepper.Header>
              <Stepper.Icon>
                <TfiPackage className="h-5 w-5" />
              </Stepper.Icon>
              <Stepper.Title>Choose the account</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {({ setStep }) => (
                <>
                  <div>
                    <Label className="text-sm" htmlFor="awsName">
                      Account Name
                    </Label>
                    <p className="pb-2 text-gray-500 text-sm">
                      Enter the name of the account you want to connect
                    </p>
                    <Input
                      className="max-w-96"
                      id="awsName"
                      onChange={(e) => setAwsName(e.target.value)}
                      value={awsName}
                    />
                  </div>
                  <div className="flex flex-col space-y-1">
                    <div className="font-medium text-sm">AWS account ID</div>
                    <p className="text-gray-500 text-sm">
                      Enter the Account ID of the one you want to connect
                    </p>
                  </div>
                  <div className="flex flex-row gap-4">
                    <Input
                      className="max-w-96"
                      onChange={(e) => setAwsAccount(e.target.value)}
                      placeholder="Enter the Account ID (123456789012)"
                      value={awsAccount}
                    />
                    <Button
                      className="w-32"
                      disabled={awsAccount.length !== 12}
                      onClick={() => setStep(1)}
                    >
                      <FaArrowRightLong className="h-4 w-4" />
                      Continue
                    </Button>
                  </div>
                </>
              )}
            </Stepper.Body>
          </Stepper.Item>
          <Stepper.Item>
            <Stepper.Header>
              <Stepper.Icon>
                <MdChecklist className="h-5 w-5" />
              </Stepper.Icon>
              <Stepper.Title>Agreement</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {({ setStep }) => (
                <>
                  <div className="flex flex-col space-y-1">
                    <div className="font-medium text-sm">
                      Brume will operate on your account
                    </div>
                    <p className="text-gray-500 text-sm">
                      Brume will operate on your account, only managing its
                      ressources. You are responsible for the stack you are
                      deploying.
                    </p>
                  </div>
                  <div className="pt-4">
                    <div className="flex items-center gap-3">
                      <Checkbox
                        checked={agreement}
                        id="terms"
                        onCheckedChange={(checked) => {
                          setAgreement(
                            checked === "indeterminate" ? false : checked
                          );
                          if (checked) {
                            setStep(2);
                          } else {
                            setStep(1);
                          }
                        }}
                      />
                      <Label htmlFor="terms">
                        Accept the changes and the risks
                      </Label>
                    </div>
                  </div>
                </>
              )}
            </Stepper.Body>
          </Stepper.Item>
          <Stepper.Item>
            <Stepper.Header>
              <Stepper.Icon>
                <MdOutlineLocalPolice className="h-5 w-5" />
              </Stepper.Icon>
              <Stepper.Title>Authorizing Brume Cloud</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {({ setStep }) => (
                <>
                  <div className="flex flex-col space-y-1">
                    <div className="font-medium text-sm">
                      Creating the Assume role
                    </div>
                    <p className="text-gray-500 text-sm">
                      Brume will use an assumed role on your account to manage
                      the ressources it deploy.
                    </p>
                  </div>
                  <div className="pt-4">
                    <Button
                      onClick={() => {
                        setStep(3);
                        redirectToAWS();
                      }}
                    >
                      Create the role
                    </Button>
                  </div>
                </>
              )}
            </Stepper.Body>
          </Stepper.Item>
          <Stepper.Item className="h-full">
            <Stepper.Header>
              <Stepper.Icon>
                <MdPreview className="h-5 w-5" />
              </Stepper.Icon>
              <Stepper.Title>Verification</Stepper.Title>
            </Stepper.Header>
            <Stepper.Body>
              {({ setStep }) => (
                <>
                  <div className="flex flex-col space-y-1">
                    <div className="font-medium text-sm">
                      Deploying the end to end test stack
                    </div>
                    <p className="text-gray-500 text-sm">
                      Brume will <span className="font-medium">dry-run</span>{" "}
                      some calls on your account to verify everything is working
                    </p>
                  </div>
                  <div className="pt-4">
                    <Button
                      disabled={accountCreationLoading}
                      onClick={() => {
                        setStep(4);
                        beginEnrollingAccount();
                      }}
                    >
                      Begin enrolling the account
                    </Button>
                  </div>
                </>
              )}
            </Stepper.Body>
          </Stepper.Item>
        </Stepper.Root>
      </Page.Body>
    </Page.Container>
  );
};
