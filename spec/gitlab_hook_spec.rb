require "spec_helper"
require "gitlab_hooks"

RSpec.describe GitlabHooks do
  before(:each) do
    allow(described_class).to receive(:forward).and_return(nil)
  end

  context "push hook" do
    let(:gl_push_zero) { ObjectifiedHash.new(total_commits_count: 0) }
    let(:gl_push) { objectified_fixture("gl_push.json") }

    it "accepts push with at least one commit" do
      expect { described_class.push_hook(gl_push) }
        .not_to raise_error
    end

    it "rejects push with zero commits" do
      expect { described_class.push_hook(gl_push_zero) }
        .to raise_error(GitlabHooks::Unsupported)
    end
  end

  context "merge request hook" do
    let(:gl_mr_open) { objectified_fixture("gl_mr_open.json") }
    let(:gl_mr_close) { objectified_fixture("gl_mr_close.json") }
    let(:gl_mr_merge) { objectified_fixture("gl_mr_merge.json") }
    let(:gl_mr_invalid) { ObjectifiedHash.new(object_attributes: { state: "none", action: "none" }) }

    it "accepts opened, closed and merged requests" do
      expect { described_class.merge_request_hook(gl_mr_open) }
        .not_to raise_error
      expect { described_class.merge_request_hook(gl_mr_close) }
        .not_to raise_error
      expect { described_class.merge_request_hook(gl_mr_merge) }
        .not_to raise_error
    end

    # are there any?
    it "rejects other requests" do
      expect { described_class.merge_request_hook(gl_mr_invalid) }
        .to raise_error(GitlabHooks::Unsupported)
    end
  end

  context "issue hook" do
    let(:gl_issue_open) { objectified_fixture("gl_issue_open.json") }
    let(:gl_issue_close) { objectified_fixture("gl_issue_close.json") }
    let(:gl_issue_invalid) { ObjectifiedHash.new(object_attributes: { state: "open", action: "update" }) }

    it "accepts opened and closed issues" do
      expect { described_class.issue_hook(gl_issue_open) }
        .not_to raise_error
      expect { described_class.issue_hook(gl_issue_close) }
        .not_to raise_error
    end

    it "rejects other issues" do
      expect { described_class.issue_hook(gl_issue_invalid) }
        .to raise_error(GitlabHooks::Unsupported)
    end
  end

  context "pipeline hook" do
    let(:gl_pipeline_success) { objectified_fixture("gl_pipeline_success.json") }
    let(:gl_pipeline_failed) { objectified_fixture("gl_pipeline_failed.json") }
    let(:gl_pipeline_reject) do
      %w[pending running].map { |s| ObjectifiedHash.new(object_attributes: { status: s }) }
    end

    it "accepts success and failed states" do
      expect { described_class.pipeline_hook(gl_pipeline_success) }
        .not_to raise_error
      expect { described_class.pipeline_hook(gl_pipeline_failed) }
        .not_to raise_error
    end

    it "rejects pending and running states" do
      gl_pipeline_reject.each do |state|
        expect { described_class.pipeline_hook(state) }
          .to raise_error(GitlabHooks::Unsupported)
      end
    end
  end
end
